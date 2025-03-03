use crate::calendar::{CalendarEvent, CalendarEventKey, TimelyDataRecord};
use crate::contract::packets::{
    CalendarEventsMetaPacket, DropCalendarEventPacket, ReferenceCalendarEventPacket,
    ReferenceDataPacket, ReferenceDataPacketType, ReferenceLocationPacket, ReferenceTimePacket,
    ReferenceTimelyDataPacket,
};
use crate::error::Error;
use log::{error, info};
use std::ops::Add;
use std::sync::Arc;
use time::{OffsetDateTime, UtcOffset};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::Duration;

use crate::commands::Commands;
use crate::events::Events;

use crate::message_bus::{BusHandler, BusSender, MessageBus};

pub struct ReferenceTime {}

pub struct Context {
    tx: Sender<Events>,
}

pub struct ProcessingContext {
    now_opt: Option<OffsetDateTime>,
    unprocessed_event_updates: Vec<ReferenceDataPacket>,
    unprocessed_event_drops: Vec<ReferenceDataPacket>,
    unprocessed_timely_data: Vec<ReferenceDataPacket>,
    update_events_count: u16,
    drop_events_count: u16,
    timely_data_count: u16,
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

        let (tx, rx) = channel::<Events>(30);

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
            unprocessed_event_updates: vec![],
            unprocessed_event_drops: vec![],
            unprocessed_timely_data: vec![],
            update_events_count: 0,
            drop_events_count: 0,
            timely_data_count: 0,
        };

        loop {
            info!("reference data loop");

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
                    ReferenceDataPacketType::CalendarEventsMeta => {
                        let deserialize_result =
                            rmp_serde::from_slice(&reference_data.packet_payload);
                        if let Err(err) = deserialize_result {
                            error!("{}", err);
                            return;
                        }

                        let events_meta: CalendarEventsMetaPacket = deserialize_result.unwrap();

                        context.update_events_count = events_meta.update_events_count;
                        context.drop_events_count = events_meta.drop_events_count;
                        context.timely_data_count = events_meta.timely_data_count;

                        info!("expecting:");
                        info!("\t{} events", context.update_events_count);
                        info!("\t{} drops", context.drop_events_count);
                        info!("\t{} timely records", context.timely_data_count);
                    }
                    ReferenceDataPacketType::CalendarEvent => {
                        context.unprocessed_event_updates.push(reference_data);

                        if Self::is_reference_data_ready(&context) {
                            Self::handle_sync_completed(context, bus);
                        }
                    }
                    ReferenceDataPacketType::DropCalendarEvent => {
                        context.unprocessed_event_drops.push(reference_data);

                        if Self::is_reference_data_ready(&context) {
                            Self::handle_sync_completed(context, bus);
                        }
                    }
                    ReferenceDataPacketType::TimelyData => {
                        context.unprocessed_timely_data.push(reference_data);

                        if Self::is_reference_data_ready(&context) {
                            Self::handle_sync_completed(context, bus);
                        }
                    }
                    _ => {}
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

    fn handle_sync_completed(context: &mut ProcessingContext, bus: &MessageBus) {
        let offset_seconds = context.now_opt.unwrap().offset();

        if context.unprocessed_event_updates.len() > 0
            || context.unprocessed_event_drops.len() > 0
            || context.unprocessed_timely_data.len() > 0
        {
            Self::handle_unprocessed_event_updates(bus, context, offset_seconds);
            Self::handle_unprocessed_event_drops(bus, context);
            Self::handle_unprocessed_timely_data(bus, context);
        }

        bus.send_event(Events::InSync(true));
    }

    fn handle_unprocessed_event_updates(
        bus: &MessageBus,
        context: &mut ProcessingContext,
        offset: UtcOffset,
    ) {
        let events_iter: Vec<_> = context
            .unprocessed_event_updates
            .drain(..)
            .filter_map(|x| {
                let res: Result<ReferenceCalendarEventPacket, rmp_serde::decode::Error> =
                    rmp_serde::from_slice(&x.packet_payload);

                match res {
                    Ok(event_dto) => Some(CalendarEvent::new(&event_dto.calendar_event, offset)),
                    Err(err) => {
                        error!("{:?} {:02X?}", err, x.packet_payload);
                        None
                    }
                }
            })
            .collect();

        for chunk in events_iter.chunks(5) {
            bus.send_event(Events::ReferenceCalendarEventUpdatesBatch(Arc::new(
                chunk.to_vec(),
            )));
        }
    }

    fn handle_unprocessed_event_drops(bus: &MessageBus, context: &mut ProcessingContext) {
        let events_iter: Vec<_> = context
            .unprocessed_event_drops
            .drain(..)
            .map(|x| {
                let drop: DropCalendarEventPacket =
                    rmp_serde::from_slice(&x.packet_payload).unwrap();
                CalendarEventKey(drop.kind, drop.event_id)
            })
            .collect();

        for chunk in events_iter.chunks(5) {
            bus.send_event(Events::ReferenceCalendarEventDropsBatch(Arc::new(
                chunk.to_vec(),
            )));
        }
    }

    fn is_reference_data_ready(context: &ProcessingContext) -> bool {
        let expected_updates = context.update_events_count as usize;
        let expected_drops = context.drop_events_count as usize;
        let expected_timely_records = context.timely_data_count as usize;

        return expected_updates == context.unprocessed_event_updates.len()
            && expected_drops == context.unprocessed_event_drops.len()
            && expected_timely_records == context.unprocessed_timely_data.len();
    }

    fn handle_unprocessed_timely_data(bus: &MessageBus, context: &mut ProcessingContext) {
        let events_iter: Vec<_> = context
            .unprocessed_timely_data
            .drain(..)
            .map(|x| {
                let packet: ReferenceTimelyDataPacket =
                    rmp_serde::from_slice(&x.packet_payload).unwrap();

                TimelyDataRecord {
                    linked_event_id: packet.linked_event_id,
                    start_at_hour: packet.start_at_hour,
                    duration: time::Duration::hours(packet.duration_hours as i64),
                    value: packet.value,
                    data_marker: packet.data_marker,
                }
            })
            .collect();

        for chunk in events_iter.chunks(5) {
            bus.send_event(Events::ReferenceTimelyDataBatch(Arc::new(chunk.to_vec())));
        }
    }
}
