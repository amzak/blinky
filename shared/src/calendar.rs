use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset};

use crate::domain::ReferenceTimeUtc;

#[derive(Clone, Debug)]
pub struct CalendarEvent {
    pub id: i64,
    pub title: String,
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CalendarEventDto {
    pub id: i64,
    pub title: String,
    pub start: ReferenceTimeUtc,
    pub end: ReferenceTimeUtc,
}

impl From<CalendarEvent> for CalendarEventDto {
    fn from(value: CalendarEvent) -> Self {
        CalendarEventDto {
            id: value.id,
            title: value.title,
            start: value.start.into(),
            end: value.end.into(),
        }
    }
}

impl Hash for CalendarEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl CalendarEvent {
    pub fn new(dto: CalendarEventDto, tz: UtcOffset) -> CalendarEvent {
        CalendarEvent {
            id: dto.id,
            start: dto.start.to_offset_dt(tz),
            end: dto.end.to_offset_dt(tz),
            title: dto.title,
        }
    }
}

impl From<CalendarEventDto> for CalendarEvent {
    fn from(dto: CalendarEventDto) -> Self {
        CalendarEvent {
            id: dto.id,
            start: dto.start.into(),
            end: dto.end.into(),
            title: dto.title,
        }
    }
}

impl PartialEq<Self> for CalendarEvent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for CalendarEvent {}
