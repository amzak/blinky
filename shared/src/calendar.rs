use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
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

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone, Copy, Hash)]
#[repr(u8)]
pub enum CalendarKind {
    Unknown = 0,
    Phone = 1,
    Trains = 2,
    Weather = 3,
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
}

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
            id: dto.id as i32,
            kind: dto.kind,
            start: dto.start.to_offset_dt(tz),
            end: dto.end.to_offset_dt(tz),
            title: dto.title,
            icon: dto.icon,
            color: dto.color,
            description: dto.description,
        }
    }

    pub fn key(&self) -> CalendarEventKey {
        CalendarEventKey(self.kind, self.id)
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
