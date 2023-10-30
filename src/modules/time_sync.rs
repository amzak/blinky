#![feature(async_closure)]

use log::{debug, info, trace};
use time::{Duration, OffsetDateTime, PrimitiveDateTime, UtcOffset};
use tokio::sync::broadcast::{Sender, Receiver};
use crate::peripherals::hal::{Commands, Events};
use crate::peripherals::nvs_storage::NvsStorage;
use crate::peripherals::rtc::Rtc;

#[derive(PartialEq)]
pub enum RtcSyncStatus {
    Init,
    InSync,
    AwaitingTimeNow,
    AwaitingReferenceTime,
    Aborted
}

pub struct RtcSync {
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

type Error<'a> = &'a str;

impl RtcSync
{
    pub async fn start(commands: Sender<Commands>, events: Sender<Events>) {
        debug!("time_sync module start...");

        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let mut storage = NvsStorage::create(NVS_NAMESPACE);

        let mut state: RtcSyncStatus = RtcSyncStatus::Init;
        let mut state_timezone = 0;

        //let init_sync_info = RtcSyncInfo::default();
        //storage.write(NVS_FIELD, &init_sync_info).unwrap();

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    match command {
                        Commands::SyncRtc => {
                            info!("{:?}", command);
                            commands.send(Commands::GetTimeNow).unwrap();
                            state = RtcSyncStatus::AwaitingTimeNow;
                        }
                        _ => {}
                    }
                },
                Ok(event) = recv_event.recv() => {
                    match event {
                        Events::TimeNow(now) => {
                            info!("{:?}", event);
                            if state != RtcSyncStatus::AwaitingTimeNow {
                                continue;
                            }

                            let mut is_in_sync = false;

                            if let Ok(sync_info) = storage.read::<RtcSyncInfo>(NVS_FIELD)
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
                                state = RtcSyncStatus::InSync;
                            }
                            else {
                                commands.send(Commands::GetReferenceTime).unwrap();
                                state = RtcSyncStatus::AwaitingReferenceTime;
                            }
                        }
                        Events::ReferenceTime(now) => {
                            info!("{:?}", event);
                            commands.send(Commands::SetTime(now)).unwrap();

                            let sync_info = RtcSyncInfo {
                                in_sync: true,
                                last_sync: now.unix_timestamp(),
                                offset: now.offset().whole_seconds()
                            };

                            storage.write(NVS_FIELD, &sync_info).unwrap();
                            state = RtcSyncStatus::InSync;

                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        info!("time_sync module stop");
    }
}