use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    calendar::CalendarEventDto,
    domain::{GpsCoordinates, ReferenceTimeOffset},
};

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
#[repr(u16)]
pub enum ReferenceDataPacketType {
    Time = 1,
    Location = 2,
    CalendarEvent = 3,
    SyncCompleted = 100,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ReferenceDataPacket {
    pub version: i32,
    pub packet_type: ReferenceDataPacketType,
    pub packet_payload_size: i32,

    #[serde(with = "serde_bytes")]
    pub packet_payload: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ReferenceTimePacket {
    pub time: ReferenceTimeOffset,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct ReferenceLocationPacket {
    pub coordinates: GpsCoordinates,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct ReferenceCalendarEventPacket {
    pub calendar_event: CalendarEventDto,
}
