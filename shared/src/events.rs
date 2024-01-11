use crate::calendar::CalendarEvent;
use crate::domain::{ReferenceData, TouchPosition, WakeupCause};
use crate::persistence::PersistenceUnit;
use time::OffsetDateTime;

#[derive(Clone, Debug)]
pub enum Events {
    TimeNow(OffsetDateTime),
    Timezone(i32),
    BluetoothConnected,
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
    ReferenceCalendarEventsCount(i32),
    CalendarEvent(CalendarEvent),
    Restored(PersistenceUnit),
    Term,
}
