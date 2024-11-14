use crate::{
    persistence::{PersistenceUnit, PersistenceUnitKind},
    reminders::Reminder,
};
use time::OffsetDateTime;

#[derive(Clone, Debug)]
pub enum Commands {
    RequestReferenceData,
    SyncCalendar,
    GetTimeNow,
    GetReferenceTime,
    SetTime(OffsetDateTime),
    StartDeepSleep,
    PauseRendering,
    ResumeRendering,
    GetTemperature,
    Persist(PersistenceUnit),
    Restore(PersistenceUnitKind),
    SetTimezone(i32),
    AbortSleep,
    ShutdownBle,
    SetReminders(Vec<Reminder>),
    DebugAccel,
    HandleAlarm,
}
