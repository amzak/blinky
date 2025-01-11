use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset};

use crate::{
    calendar::CalendarEventDto,
    reference_data::{GpsCoordinates, ReferenceTimeOffset},
};

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

#[derive(Clone, Debug)]
pub struct TouchPosition {
    pub x: i32,
    pub y: i32,
}
