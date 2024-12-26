use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
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

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct GpsCoordinates {
    pub lat: f32,
    pub lon: f32,
}
