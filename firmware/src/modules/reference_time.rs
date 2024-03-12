use blinky_shared::calendar::CalendarEvent;
use blinky_shared::contract::packets::{
    ReferenceCalendarEventPacket, ReferenceDataPacket, ReferenceDataPacketType,
    ReferenceLocationPacket, ReferenceTimePacket,
};
use blinky_shared::error::Error;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::sync::Arc;
use time::{OffsetDateTime, UtcOffset};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::Duration;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;

use blinky_shared::message_bus::{BusHandler, BusSender, MessageBus};

pub struct ReferenceTime {}

pub struct Context {
    tx: Sender<Events>,
}

pub struct ProcessingContext {
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
    async fn event_handler(_bus: &BusSender, context: &mut Context, event: Events) {
        match event {
            Events::IncomingData(_) => {
                context.tx.send(event).await.unwrap();
            }
            _ => {}
        }
    }

    async fn command_handler(bus: &BusSender, context: &mut Context, command: Commands) {
        match command {
            Commands::GetReferenceTime => {
                bus.send_cmd(Commands::RequestReferenceData);
            }
            Commands::StartDeepSleep => {
                context.tx.send(Events::Term).await.unwrap();
            }
            _ => {}
        }
    }
}

impl ReferenceTime {
    pub async fn start(bus: MessageBus) {
        info!("starting...");

        let (tx, rx) = channel::<Events>(16);

        let context = Context { tx };

        let message_bus = bus.clone();
        let processing_loop_task = tokio::task::spawn_blocking(|| {
            Self::reference_processing_loop(message_bus, rx);
        });

        MessageBus::handle::<Context, Self>(bus, context).await;

        processing_loop_task.await.unwrap();

        info!("done.");
    }

    fn reference_processing_loop(bus: MessageBus, mut rx: Receiver<Events>) {
        let mut context = ProcessingContext {
            now_opt: None,
            unprocessed_events: vec![],
        };

        loop {
            match rx.blocking_recv() {
                Some(event) => {
                    if matches!(event, Events::Term) {
                        info!("received term");
                        break;
                    }

                    Self::handle_incoming_data(&bus, &mut context, event);
                }
                None => {
                    break;
                }
            };
        }

        info!("processing loop done.");
    }

    fn handle_incoming_data(bus: &MessageBus, context: &mut ProcessingContext, event: Events) {
        match event {
            Events::IncomingData(data) => {
                let deserialize_result = rmp_serde::from_slice(&data);
                if let Err(err) = deserialize_result {
                    error!("{}", err);
                    return;
                }

                let reference_data: ReferenceDataPacket = deserialize_result.unwrap();

                match reference_data.packet_type {
                    ReferenceDataPacketType::Time => {
                        let now_result =
                            Self::handle_reference_time(bus, reference_data.packet_payload);

                        if let Err(error) = now_result {
                            error!("{}", error);
                            return;
                        }

                        context.now_opt = Some(now_result.unwrap());
                    }
                    ReferenceDataPacketType::Location => {
                        Self::handle_reference_location(bus, reference_data.packet_payload);
                    }
                    ReferenceDataPacketType::CalendarEvent => {
                        context.unprocessed_events.push(reference_data);
                    }
                    ReferenceDataPacketType::SyncCompleted => {
                        if context.now_opt.is_none() {
                            error!("no timezone info to complete sync");
                            return;
                        }

                        Self::handle_sync_completed(context, bus);
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_reference_time(bus: &MessageBus, data: Vec<u8>) -> Result<OffsetDateTime, Error> {
        let deserialize_result = rmp_serde::from_slice(&data);
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

    fn handle_reference_location(_bus: &MessageBus, data: Vec<u8>) {
        let deserialize_result = rmp_serde::from_slice(&data);
        if let Err(err) = deserialize_result {
            error!("{}", err);
            return;
        }

        let reference_location: ReferenceLocationPacket = deserialize_result.unwrap();

        info!("{:?}", reference_location)
    }

    fn handle_unprocessed_events(
        bus: &MessageBus,
        context: &mut ProcessingContext,
        offset: UtcOffset,
    ) {
        let events_iter: Vec<_> = context
            .unprocessed_events
            .drain(..)
            .map(|x| {
                let reference_calendar_event: ReferenceCalendarEventPacket =
                    rmp_serde::from_slice(&x.packet_payload).unwrap();
                CalendarEvent::new(reference_calendar_event.calendar_event, offset)
            })
            .collect();

        for chunk in events_iter.chunks(5) {
            bus.send_event(Events::ReferenceCalendarEventBatch(Arc::new(
                chunk.to_vec(),
            )));
        }
    }

    fn handle_sync_completed(context: &mut ProcessingContext, bus: &MessageBus) {
        let offset_seconds = context.now_opt.unwrap().offset();
        bus.send_cmd(Commands::ShutdownBle);
        if context.unprocessed_events.len() > 0 {
            Self::handle_unprocessed_events(bus, context, offset_seconds);
        }

        bus.send_event(Events::InSync(true));
    }
}
