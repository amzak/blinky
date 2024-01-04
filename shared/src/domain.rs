use time::OffsetDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct ReferenceTimeOffset {
    pub now: i64,
    pub offset_seconds: i32,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct GpsCoordinates {
    pub lat: f32,
    pub lon: f32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CalendarEventDto {
    pub id: i64,
    pub title: String,
    pub start: ReferenceTimeUtc,
    pub end: ReferenceTimeUtc,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct ReferenceData {
    pub version: i32,
    pub reference_time: ReferenceTimeOffset,
    pub coordinates: GpsCoordinates,
    pub events: Vec<CalendarEventDto>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum WakeupCause {
    Undef,
    All,
    Ext0,
    Ext1,
    Timer,
    Touch,
    Ulp,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ReferenceTimeUtc {
    pub unix_epoch_seconds: i64,
}

#[derive(Clone, Debug)]
pub struct TouchPosition {
    pub x: i32,
    pub y: i32,
}


#[derive(Clone, Debug)]
pub struct CalendarEvent {
    pub id: i64,
    pub title: String,
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
}