use blinky_shared::{
    calendar::CalendarKind,
    contract::packets::{
        CalendarEventSyncResponsePacket, ReferenceDataPacket, ReferenceDataPacketType,
        ReferenceTimePacket,
    },
    reference_data::ReferenceTimeOffset,
};
use serde::de::value::BytesDeserializer;
use time::OffsetDateTime;

#[test]
fn should_deserialize_reference_time_packet() {
    let offset_date_time = OffsetDateTime::now_utc();

    let expected = ReferenceTimePacket {
        time: ReferenceTimeOffset {
            now: offset_date_time.unix_timestamp(),
            offset_seconds: 60 * 60 * 2,
        },
    };

    let buf = rmp_serde::to_vec(&expected).unwrap();

    let reference_data_packet = ReferenceDataPacket {
        packet_type: blinky_shared::contract::packets::ReferenceDataPacketType::Time,
        version: 2,
        packet_payload_size: buf.len() as i32,
        packet_payload: buf,
    };

    let full_buf = rmp_serde::to_vec(&reference_data_packet).unwrap();

    let deserialize_result = rmp_serde::from_slice(&full_buf);

    let reference_data: ReferenceDataPacket = deserialize_result.unwrap();

    let deserialize_result = rmp_serde::from_slice(reference_data.packet_payload.as_slice());

    let reference_time: ReferenceTimePacket = deserialize_result.unwrap();

    assert_eq!(expected.time, reference_time.time);
}

#[test]
fn should_serialize_CalendarEventSyncResponsePacket() {
    let inner_packet = CalendarEventSyncResponsePacket {
        kind: CalendarKind::Phone,
        event_id: 1,
    };

    let buf_inner = rmp_serde::to_vec(&inner_packet).unwrap();
    let buf_inner_str = format!("{:02X?}", buf_inner);

    let reference_packet = ReferenceDataPacket::wrap(
        ReferenceDataPacketType::CalendarEventsSyncResponse,
        inner_packet,
    );

    let buf = reference_packet.serialize();
    let buf_str = format!("{:02X?}", buf);

    assert_eq!(buf_inner_str, "[92, 01, 01]");

    assert_eq!(buf_str, "[94, 02, 05, 03, C4, 03, 92, 01, 01]")
}

#[test]
fn should_() {
    let buf = [
        0x94, 0x02, 0x03, 0x16, 0xC4, 0x16, 0x91, 0x97, 0x01, 0x01, 0xA3, 0x51, 0x51, 0x51, 0x91,
        0xCE, 0x66, 0x33, 0x1F, 0xFF, 0x91, 0xCE, 0x66, 0x33, 0x1F, 0xFF, 0x02, 0x0B,
    ];

    let deserialize_result = rmp_serde::from_slice(&buf);

    let reference_data: ReferenceDataPacket = deserialize_result.unwrap();

    assert_eq!(
        reference_data.packet_type,
        ReferenceDataPacketType::CalendarEvent
    );
}
