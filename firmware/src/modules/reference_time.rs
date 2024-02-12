use blinky_shared::calendar::{CalendarEvent, CalendarEventDto};
use blinky_shared::contract::packets::{
    ReferenceCalendarEventPacket, ReferenceDataPacket, ReferenceDataPacketType,
    ReferenceLocationPacket, ReferenceTimePacket,
};
use blinky_shared::error::Error;
use log::{error, info, warn};
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

impl ReferenceTime {
    pub async fn start(commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let mut now_opt: Option<OffsetDateTime> = None;

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    match command {
                        Commands::GetReferenceTime => {
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

                            let reference_data: ReferenceDataPacket  = deserialize_result.unwrap();

                            info!("{:?}", reference_data);

                            match reference_data.packet_type {
                                ReferenceDataPacketType::Time =>  {
                                    let now_result = Self::handle_reference_time(&events, reference_data.packet_payload);
                                    if let Err(error) = now_result {
                                        error!("{}", error);
                                        continue;
                                    }

                                    now_opt = Some(now_result.unwrap());
                                },
                                ReferenceDataPacketType::Location => {
                                    Self::handle_reference_location(&events, reference_data.packet_payload);
                                },
                                ReferenceDataPacketType::CalendarEvent => {
                                    if now_opt.is_none() {
                                        warn!("calendar event skipped");
                                    }

                                    let offset_seconds = now_opt.unwrap().offset();
                                    Self::handle_reference_calendar_event(&events, reference_data.packet_payload, offset_seconds);
                                },
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        info!("done.");
    }

    fn handle_reference_time(
        events: &Sender<Events>,
        data: &[u8],
    ) -> Result<OffsetDateTime, Error> {
        let deserialize_result = rmp_serde::from_slice(data);
        if let Err(err) = deserialize_result {
            return Err(Error::from(err.to_string().as_str()));
        }

        let reference_time: ReferenceTimePacket = deserialize_result.unwrap();

        let time = reference_time.time;

        let offset = Duration::from_secs(time.offset_seconds as u64);

        let offset_from_utc = UtcOffset::from_whole_seconds(time.offset_seconds).unwrap();

        let now = OffsetDateTime::from_unix_timestamp(time.now)
            .unwrap()
            .add(offset)
            .replace_offset(offset_from_utc);

        events.send(Events::ReferenceTime(now)).unwrap();

        return Ok(now);
    }

    fn handle_reference_location(events: &Sender<Events>, data: &[u8]) {
        let deserialize_result = rmp_serde::from_slice(data);
        if let Err(err) = deserialize_result {
            error!("{}", err);
            return;
        }

        let reference_location: ReferenceLocationPacket = deserialize_result.unwrap();
    }

    fn handle_reference_calendar_event(events: &Sender<Events>, data: &[u8], offset: UtcOffset) {
        let deserialize_result = rmp_serde::from_slice(data);
        if let Err(err) = deserialize_result {
            error!("{}", err);
            return;
        }

        let reference_calendar_event: ReferenceCalendarEventPacket = deserialize_result.unwrap();

        let calendar_event = CalendarEvent::new(reference_calendar_event.calendar_event, offset);

        events
            .send(Events::ReferenceCalendarEvent(calendar_event))
            .unwrap();
    }
}
