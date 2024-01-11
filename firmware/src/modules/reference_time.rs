use blinky_shared::calendar::{CalendarEvent, CalendarEventDto};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::ops::Add;
use time::{OffsetDateTime, UtcOffset};
use tokio::sync::broadcast::Sender;
use tokio::time::Duration;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;

pub struct ReferenceTime {}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct GpsCoordinates {
    pub lat: f32,
    pub lon: f32,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct ReferenceTimeOffset {
    pub now: i64,
    pub offset_seconds: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ReferenceTimeUtc {
    pub unix_epoch_seconds: i64,
}

impl From<OffsetDateTime> for ReferenceTimeUtc {
    fn from(value: OffsetDateTime) -> Self {
        ReferenceTimeUtc {
            unix_epoch_seconds: value.unix_timestamp(),
        }
    }
}

impl Into<OffsetDateTime> for ReferenceTimeUtc {
    fn into(self) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(self.unix_epoch_seconds).unwrap()
    }
}

impl ReferenceTimeUtc {
    pub fn to_offset_dt(self, tz: UtcOffset) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(self.unix_epoch_seconds + tz.whole_seconds() as i64)
            .unwrap()
            .replace_offset(tz)
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct ReferenceData {
    pub version: i32,
    pub reference_time: ReferenceTimeOffset,
    pub coordinates: GpsCoordinates,
    pub events: Vec<CalendarEventDto>,
}

impl ReferenceTime {
    pub async fn start(commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    match command {
                        Commands::GetReferenceTime => {
                            info!("{:?}", command);
                            commands.send(Commands::RequestReferenceData).unwrap();
                        }
                        Commands::StartDeepSleep => {
                            break;
                        }
                        _ => {}
                    }
                },
                Ok(event) = recv_event.recv() => {
                    match event {
                        Events::IncomingData(data) => {
                            let deserialize_result = rmp_serde::from_slice(&data);
                            if let Err(err) = deserialize_result {
                                error!("{}", err);
                                continue;
                            }

                            let reference_data: ReferenceData  = deserialize_result.unwrap();

                            info!("{:?}", reference_data);

                            let ReferenceData {reference_time, events: calendar_events_dtos, ..} = reference_data;

                            let offset = Duration::from_secs(reference_time.offset_seconds as u64);

                            let offset_from_utc = UtcOffset::from_whole_seconds(reference_time.offset_seconds).unwrap();

                            let now = OffsetDateTime::from_unix_timestamp(reference_time.now)
                                .unwrap()
                                .add(offset)
                                .replace_offset(offset_from_utc);

                            events.send(Events::ReferenceTime(now)).unwrap();

                            let calendar_events = calendar_events_dtos.into_iter().map(|x| CalendarEvent::new(x, offset_from_utc));

                            events.send(Events::ReferenceCalendarEventsCount(calendar_events.len() as i32)).unwrap();

                            for event in calendar_events {
                                events.send(Events::ReferenceCalendarEvent(event)).unwrap();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        info!("done.");
    }
}
