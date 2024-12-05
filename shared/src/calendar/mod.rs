use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use time::{Duration, OffsetDateTime, UtcOffset};

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
    CalendarAlert = 8,
    Alarm = 9,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone, Copy, Hash)]
#[repr(u8)]
pub enum CalendarKind {
    Unknown = 0,
    Phone = 1,
    Trains = 2,
    Weather = 3,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone, Copy, Hash)]
#[repr(u8)]
pub enum CalendarEventRemainderStatus {
    Disabled = 0,
    Planned = 1,
    Passed = 2,
}

#[derive(Clone, Debug)]
pub struct CalendarEvent {
    pub kind: CalendarKind,
    pub id: i32,
    pub title: String,
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
    pub icon: CalendarEventIcon,
    pub color: u32,
    pub description: String,
    pub lane: u8,
}

#[derive(Clone, Debug)]
pub struct CalendarEventSegment {
    pub event_id: i32,
    pub lane: u8,
    pub start: Option<OffsetDateTime>,
    pub end: Option<OffsetDateTime>,
}

#[derive(Debug)]
pub struct CalendarEventOrderedByStartAsc(pub CalendarEvent);

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CalendarEventDto {
    pub kind: CalendarKind,
    pub id: i32,
    pub title: String,
    pub start: ReferenceTimeUtc,
    pub end: ReferenceTimeUtc,
    pub icon: CalendarEventIcon,
    pub color: u32,
    pub description: String,
    pub lane: u8,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Hash)]
pub struct CalendarEventKey(pub CalendarKind, pub i32);

impl From<CalendarEvent> for CalendarEventDto {
    fn from(value: CalendarEvent) -> Self {
        CalendarEventDto {
            id: value.id as i32,
            kind: value.kind,
            title: value.title,
            start: value.start.into(),
            end: value.end.into(),
            icon: value.icon,
            color: value.color,
            description: value.description,
            lane: value.lane,
        }
    }
}

impl From<&CalendarEvent> for CalendarEventDto {
    fn from(value: &CalendarEvent) -> Self {
        CalendarEventDto {
            id: value.id as i32,
            kind: value.kind,
            title: value.title.clone(),
            start: value.start.into(),
            end: value.end.into(),
            icon: value.icon,
            color: value.color,
            description: value.description.clone(),
            lane: value.lane,
        }
    }
}

impl Hash for CalendarEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.id.hash(state);
    }
}

impl Ord for CalendarEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let order = other.start.cmp(&self.start);

        if matches!(order, std::cmp::Ordering::Equal) {
            return other.id.cmp(&self.id);
        }

        return order;
    }
}

impl PartialOrd for CalendarEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CalendarEventOrderedByStartAsc {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let order = self.0.start.cmp(&other.0.start);

        if matches!(order, std::cmp::Ordering::Equal) {
            return self.0.id.cmp(&other.0.id);
        }

        return order;
    }
}

impl PartialOrd for CalendarEventOrderedByStartAsc {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl CalendarEvent {
    pub fn new(dto: &CalendarEventDto, tz: UtcOffset) -> CalendarEvent {
        CalendarEvent {
            id: dto.id as i32,
            kind: dto.kind,
            start: dto.start.clone().to_offset_dt(tz),
            end: dto.end.clone().to_offset_dt(tz),
            title: dto.title.clone(),
            icon: dto.icon,
            color: dto.color,
            description: dto.description.clone(),
            lane: dto.lane,
        }
    }

    pub fn key(&self) -> CalendarEventKey {
        CalendarEventKey(self.kind, self.id)
    }

    pub fn duration(&self) -> Duration {
        self.end - self.start
    }
}

impl From<CalendarEventDto> for CalendarEvent {
    fn from(dto: CalendarEventDto) -> Self {
        CalendarEvent {
            id: dto.id as i32,
            kind: dto.kind,
            start: dto.start.into(),
            end: dto.end.into(),
            title: dto.title,
            icon: dto.icon,
            color: dto.color,
            description: dto.description,
            lane: dto.lane,
        }
    }
}

impl PartialEq<Self> for CalendarEvent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for CalendarEvent {}

impl Eq for CalendarEventKey {}

impl PartialEq<Self> for CalendarEventOrderedByStartAsc {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}

impl Eq for CalendarEventOrderedByStartAsc {}
