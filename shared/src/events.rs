use std::sync::Arc;

use crate::calendar::{CalendarEvent, CalendarEventKey};
use crate::domain::{ReferenceData, TouchPosition, WakeupCause};
use crate::persistence::PersistenceUnit;
use strum_macros::AsRefStr;
use time::OffsetDateTime;

#[derive(Clone, Debug, AsRefStr)]
pub enum Events {
    TimeNow(OffsetDateTime),
    BleClientConnected,
    BleClientDisconnected,
    ReferenceData(ReferenceData),
    ReferenceTime(OffsetDateTime),
    Wakeup(WakeupCause),
    TouchOrMove,
    Key1Press,
    TouchPos(TouchPosition),
    IncomingData(Arc<Vec<u8>>),
    Temperature(f32),
    BatteryLevel(u16),
    Charging(bool),
    InSync(bool),
    ReferenceCalendarEvent(CalendarEvent),
    ReferenceCalendarEventUpdatesBatch(Arc<Vec<CalendarEvent>>),
    ReferenceCalendarEventDropsBatch(Arc<Vec<CalendarEventKey>>),
    CalendarEvent(CalendarEvent),
    CalendarEventsBatch(Arc<Vec<CalendarEvent>>),
    DropCalendarEventsBatch(Arc<Vec<CalendarEventKey>>),
    Restored(PersistenceUnit),
    PersistedCalendarEvents(Arc<Vec<CalendarEventKey>>),
    FirstRender,
    Term,
}
