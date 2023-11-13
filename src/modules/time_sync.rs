use log::{debug, info};
use time::{Duration, OffsetDateTime, UtcOffset};
use tokio::sync::broadcast::Sender;
use crate::peripherals::hal::{Commands, Events};
use crate::peripherals::nvs_storage::NvsStorage;
use tokio::sync::{Mutex, watch};
use tokio::select;
use tokio::time::MissedTickBehavior;

#[derive(PartialEq, Copy, Clone)]
pub enum RtcSyncState {
    Init,
    InSync,
    AwaitingTimeNow,
    AwaitingReferenceTime,
    Aborted
}

pub struct TimeSync {
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct RtcSyncInfo {
    pub last_sync: i64,
    pub offset: i32,
    //pub last_sync_utc: OffsetDateTime,
    pub in_sync: bool
}

const NVS_NAMESPACE: &str = "rtc_sync";
const NVS_FIELD: &str = "rtc_sync_info";
const SYNC_INTERVAL_DAYS: i8 = 1;

impl TimeSync
{
    pub async fn start(commands: Sender<Commands>, events: Sender<Events>) {
        debug!("time_sync module start...");

        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let mut storage = Mutex::new(NvsStorage::create(NVS_NAMESPACE));

        let state: Mutex<RtcSyncState> = Mutex::new(RtcSyncState::Init);
        let state_timezone = 0;

        //let init_sync_info = RtcSyncInfo::default();
        //storage.write(NVS_FIELD, &init_sync_info).unwrap();

        let cm1 = commands.clone();
        let (tx, rx) = watch::channel(true);

        let timer = tokio::spawn(Self::run_timer(rx, cm1));

        loop {
            select! {
                Ok(command) = recv_cmd.recv() => {
                    info!("{:?}", command);
                    match command {
                        Commands::SyncRtc => {
                            Self::set_state(&state, RtcSyncState::AwaitingTimeNow).await;
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
                        Events::TimeNow(now) => {

                            if Self::get_state(&state).await != RtcSyncState::AwaitingTimeNow {
                                continue;
                            }

                            let mut is_in_sync = false;

                            if let Ok(sync_info) = storage
                                .lock()
                                .await
                                .read::<RtcSyncInfo>(NVS_FIELD)
                            {
                                info!("{:?}", sync_info);
                                let RtcSyncInfo {last_sync, offset, in_sync} = sync_info;

                                let last_sync_woffset = OffsetDateTime::from_unix_timestamp(last_sync).unwrap().replace_offset(UtcOffset::from_whole_seconds(offset).unwrap());

                                let diff = now - last_sync_woffset;

                                is_in_sync = in_sync && diff <= Duration::days(SYNC_INTERVAL_DAYS as i64);
                                info!("{:?} {:?}", diff, is_in_sync);
                            }

                            is_in_sync = false;

                            if is_in_sync {
                                Self::set_state(&state, RtcSyncState::InSync).await;
                            }
                            else {
                                commands.send(Commands::GetReferenceTime).unwrap();
                                Self::set_state(&state, RtcSyncState::AwaitingReferenceTime).await;
                            }
                        }
                        Events::ReferenceTime(now) => {
                            commands.send(Commands::SetTime(now)).unwrap();

                            let sync_info = RtcSyncInfo {
                                in_sync: true,
                                last_sync: now.unix_timestamp(),
                                offset: now.offset().whole_seconds()
                            };

                            storage.lock().await.write(NVS_FIELD, &sync_info).unwrap();
                            Self::set_state(&state, RtcSyncState::InSync).await;
                        }
                        _ => {}
                    }
                }
            }
        }

        timer.abort();

        info!("done.");
    }

    async fn run_timer(pause_param: watch::Receiver<bool>, commands: Sender<Commands>) {
        let mut pause = pause_param;

        let mut interval = tokio::time::interval(core::time::Duration::from_secs(1));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        info!("before loop");

        //let pause_flag: AtomicBool = AtomicBool::new(false);
        let mut pause_flag = false;

        loop {
            select! {
                Ok(flag) = pause.changed() => {
                    let val = pause.borrow_and_update();
                    pause_flag = *val;
                    //pause_flag.store(*val, Ordering::Relaxed);
                }
                _ = interval.tick() => {
                    if pause_flag { //.load(Ordering::Relaxed) {
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

    async fn set_state(state: &Mutex<RtcSyncState>, value: RtcSyncState) {
        let mut lock = state.lock().await;
        *lock = value;
    }

    async fn get_state(state: &Mutex<RtcSyncState>) -> RtcSyncState {
        let val = state.lock().await;
        return *val;
    }
}