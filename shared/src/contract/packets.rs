use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::serde_as;
use serde_with::Bytes;

use crate::calendar::TimelyDataMarker;
use crate::calendar::{CalendarEventDto, CalendarKind};
use crate::reference_data::GpsCoordinates;
use crate::reference_data::ReferenceTimeOffset;
use crate::reference_data::ReferenceTimeUtc;

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
#[repr(u16)]
pub enum ReferenceDataPacketType {
    Time = 1,
    Location = 2,
    CalendarEvent = 3,
    CalendarEventsMeta = 4,
    CalendarEventsSyncResponse = 5,
    DropCalendarEvent = 6,
    TimelyData = 7,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ReferenceDataPacket {
    pub version: i32,
    pub packet_type: ReferenceDataPacketType,
    pub packet_payload_size: i32,

    #[serde_as(as = "Bytes")]
    pub packet_payload: Vec<u8>,
}

impl ReferenceDataPacket {
    pub fn wrap<T>(packet_type: ReferenceDataPacketType, obj: T) -> Self
    where
        T: Serialize,
    {
        let buf = rmp_serde::to_vec(&obj).unwrap();

        let reference_data_packet = ReferenceDataPacket {
            version: 2,
            packet_type,
            packet_payload_size: buf.len() as i32,
            packet_payload: buf,
        };

        reference_data_packet
    }

    pub fn serialize(self) -> Vec<u8> {
        let full_buf = rmp_serde::to_vec(&self).unwrap();
        return full_buf;
    }
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

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct ReferenceTimelyDataPacket {
    pub linked_event_id: i32,
    pub start_at_hour: u8,
    pub duration_hours: u8,
    pub value: f32,
    pub data_marker: TimelyDataMarker,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct CalendarEventsMetaPacket {
    pub update_events_count: u16,
    pub drop_events_count: u16,
    pub timely_data_count: u16,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CalendarEventSyncResponsePacket {
    pub kind: CalendarKind,
    pub event_id: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct DropCalendarEventPacket {
    pub kind: CalendarKind,
    pub event_id: i32,
}
