use crate::calendar::CalendarEvent;
use crate::domain::{ReferenceData, TouchPosition, WakeupCause};
use crate::persistence::PersistenceUnit;
use strum_macros::AsRefStr;
use time::OffsetDateTime;

#[derive(Clone, Debug, AsRefStr)]
pub enum Events {
    TimeNow(OffsetDateTime),
    BluetoothConnected,
    BluetoothDisconnected,
    ReferenceData(ReferenceData),
    ReferenceTime(OffsetDateTime),
    Wakeup(WakeupCause),
    TouchOrMove,
    TouchPos(TouchPosition),
    IncomingData(Vec<u8>),
    Temperature(f32),
    BatteryLevel(u16),
    Charging(bool),
    InSync(bool),
    ReferenceCalendarEvent(CalendarEvent),
    ReferenceCalendarEventBatch(Vec<CalendarEvent>),
    CalendarEvent(CalendarEvent),
    CalendarEventsBatch(Vec<CalendarEvent>),
    Restored(PersistenceUnit),
    Term,
}
