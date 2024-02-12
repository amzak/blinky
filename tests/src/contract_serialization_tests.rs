use blinky_shared::{
    contract::packets::{ReferenceDataPacket, ReferenceTimePacket},
    domain::ReferenceTimeOffset,
};
use serde::de::value::BytesDeserializer;
use time::OffsetDateTime;

#[test]
fn should_deserialize_reference_data_packet() {
    let offset_date_time = OffsetDateTime::now_utc();

    let ref_time = ReferenceTimePacket {
        time: ReferenceTimeOffset {
            now: offset_date_time.unix_timestamp(),
            offset_seconds: 60 * 60 * 2,
        },
    };

    let buf = rmp_serde::to_vec(&ref_time).unwrap();

    let reference_data_packet = ReferenceDataPacket {
        packet_type: blinky_shared::contract::packets::ReferenceDataPacketType::Time,
        version: 2,
        packet_payload_size: buf.len() as i32,
        packet_payload: buf.as_slice(),
    };

    let full_buf = rmp_serde::to_vec(&reference_data_packet).unwrap();

    let deserialize_result = rmp_serde::from_slice(&full_buf);

    let reference_data: ReferenceDataPacket = deserialize_result.unwrap();
}
