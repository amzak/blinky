use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset};

use crate::domain::ReferenceTimeUtc;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum CalendarEventIcon {
    Default = 0,
    Meeting = 1,
    Birthday = 2,
    Trip = 3,
    Bus = 4,
    Train = 5,
    Car = 6,
    Rain = 7,
}

#[derive(Clone, Debug)]
pub struct CalendarEvent {
    pub id: i64,
    pub title: String,
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
    pub icon: CalendarEventIcon,
    pub color: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CalendarEventDto {
    pub id: i64,
    pub title: String,
    pub start: ReferenceTimeUtc,
    pub end: ReferenceTimeUtc,
    pub icon: CalendarEventIcon,
    pub color: u32,
}

impl From<CalendarEvent> for CalendarEventDto {
    fn from(value: CalendarEvent) -> Self {
        CalendarEventDto {
            id: value.id,
            title: value.title,
            start: value.start.into(),
            end: value.end.into(),
            icon: value.icon,
            color: value.color,
        }
    }
}

impl Hash for CalendarEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Ord for CalendarEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.start.cmp(&self.start)
    }
}

impl PartialOrd for CalendarEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl CalendarEvent {
    pub fn new(dto: CalendarEventDto, tz: UtcOffset) -> CalendarEvent {
        CalendarEvent {
            id: dto.id,
            start: dto.start.to_offset_dt(tz),
            end: dto.end.to_offset_dt(tz),
            title: dto.title,
            icon: dto.icon,
            color: dto.color,
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
            icon: dto.icon,
            color: dto.color,
        }
    }
}

impl PartialEq<Self> for CalendarEvent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for CalendarEvent {}
