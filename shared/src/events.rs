use time::OffsetDateTime;
use crate::domain::{CalendarEvent, ReferenceData, TouchPosition, WakeupCause};
use crate::persistence::PersistenceUnit;

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
