use std::sync::Arc;

use crate::calendar::{CalendarEvent, CalendarEventKey, EventTimelyData, TimelyDataRecord};
use crate::domain::{ReferenceData, TouchPosition, WakeupCause};
use crate::persistence::PersistenceUnit;
use crate::reminders::Reminder;
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
    SharedInterrupt, // touch, accelerometer, rtc alarm
    Key1Press,
    Key2Press,
    TouchPos(TouchPosition),
    IncomingData(Arc<Vec<u8>>),
    Temperature(i32),
    BatteryLevel(u16),
    Charging(bool),
    InSync(bool),
    ReferenceCalendarEvent(CalendarEvent),
    ReferenceCalendarEventUpdatesBatch(Arc<Vec<CalendarEvent>>),
    ReferenceCalendarEventDropsBatch(Arc<Vec<CalendarEventKey>>),
    ReferenceTimelyDataBatch(Arc<Vec<TimelyDataRecord>>),
    CalendarEvent(CalendarEvent),
    CalendarEventsBatch(Arc<Vec<CalendarEvent>>),
    TimelyDataBatch(Arc<Vec<TimelyDataRecord>>),
    DropCalendarEventsBatch(Arc<Vec<CalendarEventKey>>),
    Restored(PersistenceUnit),
    PersistedCalendarEvents(Arc<Vec<CalendarEventKey>>),
    FirstRender,
    Reminder(Reminder),
    Term,
    AccelerometerInterrupt(u8),
    RtcAlarmInterrupt(bool),
    EventTimelyData(EventTimelyData),
}
