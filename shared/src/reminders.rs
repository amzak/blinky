use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq)]
pub enum ReminderKind {
    Event,
    Alert,
    Notification,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reminder {
    pub remind_at: OffsetDateTime,
    pub kind: ReminderKind,
    pub event_id: i32,
}

impl Ord for Reminder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.remind_at.cmp(&other.remind_at)
    }
}

impl PartialOrd for Reminder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Reminder {}
