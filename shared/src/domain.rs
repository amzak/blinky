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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ReferenceTimeUtc {
    pub unix_epoch_seconds: i64,
}

#[derive(Clone, Debug)]
pub struct TouchPosition {
    pub x: i32,
    pub y: i32,
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
    pub fn to_offset_dt(self, tz: UtcOffset) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(self.unix_epoch_seconds + tz.whole_seconds() as i64)
            .unwrap()
            .replace_offset(tz)
    }
}
