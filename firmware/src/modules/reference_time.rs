use blinky_shared::calendar::{CalendarEvent, CalendarEventDto};
use blinky_shared::contract::packets::{
    ReferenceCalendarEventPacket, ReferenceDataPacket, ReferenceDataPacketType,
    ReferenceLocationPacket, ReferenceTimePacket,
};
use blinky_shared::error::Error;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::ops::Add;
use time::{OffsetDateTime, UtcOffset};
use tokio::task::JoinHandle;
use tokio::time::Duration;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;

use blinky_shared::message_bus::{BusHandler, BusSender, MessageBus};

pub struct ReferenceTime {}

pub struct Context {
    now_opt: Option<OffsetDateTime>,
    unprocessed_events: Vec<ReferenceDataPacket>,
}

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
    pub fn _to_offset_dt(self, tz: UtcOffset) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(self.unix_epoch_seconds + tz.whole_seconds() as i64)
            .unwrap()
            .replace_offset(tz)
    }
}

impl BusHandler<Context> for ReferenceTime {
    async fn event_handler(bus: &BusSender, context: &mut Context, event: Events) {
        match event {
            Events::IncomingData(data) => {
                let deserialize_result = Self::deserialize(data).await;
                if let Err(err) = deserialize_result {
                    error!("{}", err);
                    return;
                }

                let reference_data: ReferenceDataPacket = deserialize_result.unwrap();

                match reference_data.packet_type {
                    ReferenceDataPacketType::Time => {
                        let now_result =
                            Self::handle_reference_time(bus, reference_data.packet_payload).await;

                        if let Err(error) = now_result {
                            error!("{}", error);
                            return;
                        }

                        context.now_opt = Some(now_result.unwrap());
                    }
                    ReferenceDataPacketType::Location => {
                        Self::handle_reference_location(bus, reference_data.packet_payload).await;
                    }
                    ReferenceDataPacketType::CalendarEvent => {
                        context.unprocessed_events.push(reference_data);
                    }
                    ReferenceDataPacketType::SyncCompleted => {
                        let offset_seconds = context.now_opt.unwrap().offset();
                        bus.send_cmd(Commands::DisconnectBle);

                        if context.unprocessed_events.len() > 0 {
                            Self::handle_unprocessed_events(bus.clone(), context, offset_seconds)
                                .await;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    async fn command_handler(bus: &BusSender, _context: &mut Context, command: Commands) {
        match command {
            Commands::GetReferenceTime => {
                bus.send_cmd(Commands::RequestReferenceData);
            }
            _ => {}
        }
    }
}

impl ReferenceTime {
    pub async fn start(bus: MessageBus) {
        info!("starting...");

        let context = Context {
            now_opt: None,
            unprocessed_events: vec![],
        };

        MessageBus::handle::<Context, Self>(bus, context).await;

        info!("done.");
    }

    async fn handle_reference_time(
        bus: &BusSender,
        data: Vec<u8>,
    ) -> Result<OffsetDateTime, Error> {
        let deserialize_result = Self::deserialize(data).await;
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

        bus.send_event(Events::ReferenceTime(now));

        return Ok(now);
    }

    async fn handle_reference_location(_bus: &BusSender, data: Vec<u8>) {
        let deserialize_result = Self::deserialize(data).await;
        if let Err(err) = deserialize_result {
            error!("{}", err);
            return;
        }

        let _reference_location: ReferenceLocationPacket = deserialize_result.unwrap();
    }

    fn deserialize<TPacket>(data: Vec<u8>) -> JoinHandle<TPacket>
    where
        TPacket: for<'a> Deserialize<'a> + Send + 'static,
    {
        tokio::task::spawn_blocking(move || {
            let data_inner = data;
            let res = rmp_serde::from_slice(&data_inner).unwrap();
            res
        })
    }

    async fn handle_unprocessed_events(bus: BusSender, context: &mut Context, offset: UtcOffset) {
        let packets: Vec<_> = context.unprocessed_events.drain(..).collect();

        tokio::task::spawn_blocking(move || {
            let mut events_iter = packets.iter().map(|x| {
                let reference_calendar_event: ReferenceCalendarEventPacket =
                    rmp_serde::from_slice(&x.packet_payload).unwrap();
                CalendarEvent::new(reference_calendar_event.calendar_event, offset)
            });

            for chunk in events_iter.next_chunk::<5>() {
                bus.send_event(Events::ReferenceCalendarEventBatch(chunk.to_vec()));
            }
        })
        .await
        .unwrap();
    }
}
