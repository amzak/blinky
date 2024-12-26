use std::{pin::Pin, sync::Arc, time::Duration};

use blinky_shared::{
    contract::packets::{ReferenceDataPacket, ReferenceDataPacketType, ReferenceTimePacket},
    events::Events,
    message_bus::MessageBus,
    modules::reference_time::ReferenceTime,
    reference_data::ReferenceTimeOffset,
};
use time::OffsetDateTime;
use tokio::{join, time::sleep};

use crate::spy_module::{SpyModule, SpyResult};

#[tokio::test]
async fn should_emit_ReferenceTime_event_to_the_message_bus() {
    let message_bus = MessageBus::new();

    let mb = message_bus.clone();
    let message_bus_clone = message_bus.clone();

    let some_time = get_some_time();

    let reference_time_task = ReferenceTime::start(mb);

    let time_clone = some_time.clone();

    let startup_sequence = async move {
        let packet = some_reference_time_packet(some_time);
        let buf = packet.serialize();

        message_bus.send_event(Events::IncomingData(Arc::new(buf)));
    };

    let reference_time = some_refetence_time_event();

    let mut spy = SpyModule::new();

    let spy_task = spy.start(message_bus_clone, reference_time);

    let tasks: Vec<Pin<Box<dyn futures::Future<Output = ()>>>> = vec![
        Box::pin(reference_time_task),
        Box::pin(startup_sequence),
        Box::pin(spy_task),
    ];

    futures::future::join_all(tasks).await;

    let mut result = spy.get_result();

    let first = result.next();

    let Events::ReferenceTime(result_ref_time) = first.unwrap() else {
        assert!(false, "failed to get first item");
        return;
    };

    let timestamp_expected = time_clone.now;
    let offset_expected = time_clone.offset_seconds;

    assert_eq!(result_ref_time.unix_timestamp(), timestamp_expected);
    assert_eq!(result_ref_time.offset().whole_seconds(), offset_expected);
}

fn get_some_time() -> ReferenceTimeOffset {
    let offset_date_time = OffsetDateTime::now_utc();

    let time = ReferenceTimeOffset {
        now: offset_date_time.unix_timestamp(),
        offset_seconds: 60 * 60 * 2,
    };

    time
}

fn some_reference_time_packet(time: ReferenceTimeOffset) -> ReferenceDataPacket {
    let packet = ReferenceTimePacket { time };

    ReferenceDataPacket::wrap(ReferenceDataPacketType::Time, packet)
}

fn some_refetence_time_event() -> Events {
    let offset_date_time = OffsetDateTime::now_utc();
    Events::ReferenceTime(offset_date_time)
}
