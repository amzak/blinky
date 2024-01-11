use blinky_shared::persistence::{PersistenceUnit, PersistenceUnitKind};
use log::{debug, error, info};
use time::{Duration, OffsetDateTime, UtcOffset};
use tokio::select;
use tokio::sync::broadcast::Sender;
use tokio::sync::watch;
use tokio::time::MissedTickBehavior;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;

pub struct TimeSync {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, Hash)]
pub struct RtcSyncInfo {
    pub last_sync: i64,
    pub offset: i32,
    pub in_sync: bool,
}

impl Into<OffsetDateTime> for &RtcSyncInfo {
    fn into(self) -> OffsetDateTime {
        let last_sync = OffsetDateTime::from_unix_timestamp(self.last_sync)
            .unwrap()
            .replace_offset(UtcOffset::from_whole_seconds(self.offset).unwrap());
        last_sync
    }
}

const SYNC_INTERVAL_MINUTES: i8 = 10;

impl TimeSync {
    pub async fn start(commands: Sender<Commands>, events: Sender<Events>) {
        debug!("time_sync module start...");

        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let cm1 = commands.clone();
        let (tx, rx) = watch::channel(true);

        let timer = tokio::spawn(Self::run_timer(rx, cm1));

        let mut now: Option<OffsetDateTime> = None;
        let mut sync_info: Option<RtcSyncInfo> = None;

        loop {
            select! {
                Ok(command) = recv_cmd.recv() => {
                    info!("{:?}", command);
                    match command {
                        Commands::SyncRtc => {
                            commands
                                .send(Commands::Restore(PersistenceUnitKind::RtcSyncInfo))
                                .unwrap();
                        }
                        Commands::StartDeepSleep => {
                            tx.send(true).unwrap();
                            break;
                        }
                        Commands::PauseRendering => {
                            tx.send(true).unwrap();
                        }
                        Commands::ResumeRendering => {
                            tx.send(false).unwrap();
                        }
                        _ => {}
                    }
                },
                Ok(event) = recv_event.recv() => {
                    info!("{:?}", event);
                    match event {
                        Events::TimeNow(time) => {
                            if now.is_some() {
                                continue;
                            }

                            now = Some(time);

                            if Self::is_sync_required(&now, &sync_info) {
                                commands.send(Commands::GetReferenceTime).unwrap();
                            }
                        }
                        Events::ReferenceTime(now) => {
                            commands.send(Commands::SetTime(now)).unwrap();

                            let rtc_sync_info = RtcSyncInfo {
                                in_sync: true,
                                last_sync: now.unix_timestamp(),
                                offset: now.offset().whole_seconds()
                            };

                            let unit = PersistenceUnit::new(PersistenceUnitKind::RtcSyncInfo, &rtc_sync_info);
                            commands.send(Commands::Persist(unit)).unwrap();
                        }
                        Events::Restored(unit) => {
                            if !matches!(unit.kind, PersistenceUnitKind::RtcSyncInfo) {
                                continue;
                            }

                            if let Err(error) = unit.data {
                                error!("{}", error);
                                commands.send(Commands::GetReferenceTime).unwrap();
                                continue;
                            }

                            let res = unit.deserialize();

                            match res {
                                Ok(sync_info_restored) => {
                                    info!("{:?}", sync_info_restored);

                                    sync_info = Some(sync_info_restored);

                                    let utc_offset = sync_info.as_ref().unwrap().offset;
                                    events.send(Events::Timezone(utc_offset)).unwrap();

                                    if Self::is_sync_required(&now, &sync_info) {
                                        commands.send(Commands::GetReferenceTime).unwrap();
                                    }
                                },
                                Err(error) => {
                                    error!("{:?}", error);
                                    commands.send(Commands::GetReferenceTime).unwrap();
                                    continue;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        timer.abort();

        info!("done.");
    }

    fn is_sync_required(
        now_opt: &Option<OffsetDateTime>,
        sync_info_opt: &Option<RtcSyncInfo>,
    ) -> bool {
        if now_opt.is_none() || sync_info_opt.is_none() {
            return false;
        }

        let sync_info = sync_info_opt.as_ref().unwrap();
        let now = now_opt.unwrap();
        let last_sync: OffsetDateTime = sync_info.into();

        if sync_info.in_sync && Self::is_in_sync(&now, &last_sync) {
            return false;
        }

        return true;
    }

    fn is_in_sync(now: &OffsetDateTime, last_sync: &OffsetDateTime) -> bool {
        let diff = *now - *last_sync;
        let is_in_sync = diff <= Duration::minutes(SYNC_INTERVAL_MINUTES as i64);

        info!("{:?} {:?}", diff, is_in_sync);

        is_in_sync
    }

    async fn run_timer(pause_param: watch::Receiver<bool>, commands: Sender<Commands>) {
        let mut pause = pause_param;

        let mut interval = tokio::time::interval(core::time::Duration::from_secs(1));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        info!("before loop");

        let mut pause_flag = false;

        loop {
            select! {
                Ok(flag) = pause.changed() => {
                    let val = pause.borrow_and_update();
                    pause_flag = *val;
                }
                _ = interval.tick() => {
                    if pause_flag {
                        info!("pause");
                        continue;
                    }

                    info!("tick");


                    commands.send(Commands::GetTimeNow).unwrap();
                }
            }
        }

        info!("out of tick loop");
    }
}
