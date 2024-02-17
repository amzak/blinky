use blinky_shared::persistence::{PersistenceUnit, PersistenceUnitKind};
use log::{debug, error, info};
use time::{Duration, OffsetDateTime, UtcOffset};
use tokio::select;
use tokio::sync::watch::{self, Sender};
use tokio::time::MissedTickBehavior;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;

use blinky_shared::message_bus::{BusHandler, BusSender, MessageBus};

pub struct TimeSync {}

struct Context {
    now: Option<OffsetDateTime>,
    sync_info: Option<RtcSyncInfo>,
    tx: Sender<bool>,
}

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

impl BusHandler<Context> for TimeSync {
    async fn event_handler(bus: &BusSender, context: &mut Context, event: Events) {
        match event {
            Events::TimeNow(time) => {
                if context.now.is_some() {
                    return;
                }

                context.now = Some(time);

                if Self::is_sync_required(&context.now, &context.sync_info) {
                    bus.send_cmd(Commands::GetReferenceTime);
                }
            }
            Events::ReferenceTime(now) => {
                bus.send_cmd(Commands::SetTime(now));

                let rtc_sync_info = RtcSyncInfo {
                    in_sync: true,
                    last_sync: now.unix_timestamp(),
                    offset: now.offset().whole_seconds(),
                };

                let unit = PersistenceUnit::new(PersistenceUnitKind::RtcSyncInfo, &rtc_sync_info);
                bus.send_cmd(Commands::Persist(unit));
            }
            Events::Restored(unit) => {
                if !matches!(unit.kind, PersistenceUnitKind::RtcSyncInfo) {
                    return;
                }

                if let Err(error) = unit.data {
                    error!("{}", error);
                    bus.send_cmd(Commands::GetReferenceTime);
                    return;
                }

                let res = unit.deserialize();

                match res {
                    Ok(sync_info_restored) => {
                        info!("{:?}", sync_info_restored);

                        context.sync_info = Some(sync_info_restored);

                        let utc_offset = context.sync_info.as_ref().unwrap().offset;
                        bus.send_cmd(Commands::SetTimezone(utc_offset));

                        if Self::is_sync_required(&context.now, &context.sync_info) {
                            bus.send_cmd(Commands::GetReferenceTime);
                        }
                    }
                    Err(error) => {
                        error!("{:?}", error);
                        bus.send_cmd(Commands::GetReferenceTime);
                        return;
                    }
                }
            }
            _ => {}
        }
    }

    async fn command_handler(bus: &BusSender, context: &mut Context, command: Commands) {
        match command {
            Commands::SyncRtc => {
                bus.send_cmd(Commands::Restore(PersistenceUnitKind::RtcSyncInfo));
            }
            Commands::StartDeepSleep => {
                context.tx.send(true).unwrap();
                return;
            }
            Commands::PauseRendering => {
                context.tx.send(true).unwrap();
            }
            Commands::ResumeRendering => {
                context.tx.send(false).unwrap();
            }
            _ => {}
        }
    }
}

impl TimeSync {
    pub async fn start(bus: MessageBus) {
        info!("starting...");

        let (tx, rx) = watch::channel(true);

        let timer = tokio::spawn(Self::run_timer(rx, bus.clone()));

        let context = Context {
            now: None,
            sync_info: None,
            tx,
        };

        MessageBus::handle::<Context, Self>(bus, context).await;

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

        let in_sync = sync_info.in_sync && Self::is_in_sync(&now, &last_sync);

        return !in_sync;
    }

    fn is_in_sync(now: &OffsetDateTime, last_sync: &OffsetDateTime) -> bool {
        let diff = *now - *last_sync;
        let is_in_sync = diff <= Duration::minutes(SYNC_INTERVAL_MINUTES as i64);

        info!("{:?} {:?}", diff, is_in_sync);

        is_in_sync
    }

    async fn run_timer(pause_param: watch::Receiver<bool>, bus: MessageBus) {
        let mut pause = pause_param;

        let mut interval = tokio::time::interval(core::time::Duration::from_secs(1));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        info!("before loop");

        let mut pause_flag = false;

        loop {
            select! {
                Ok(_) = pause.changed() => {
                    let val = pause.borrow_and_update();
                    pause_flag = *val;
                }
                _ = interval.tick() => {
                    if pause_flag {
                        info!("pause");
                        continue;
                    }

                    info!("tick");


                    bus.send_cmd(Commands::GetTimeNow);
                }
            }
        }

        info!("out of tick loop");
    }
}
