use blinky_shared::{
    calendar::CalendarEventDto,
    domain::ReferenceTimeUtc,
    persistence::{PersistenceUnit, PersistenceUnitKind},
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct CalendarEventDto_OldVersion {
    pub id: i64,
    pub title: String,
    pub start: ReferenceTimeUtc,
    pub end: ReferenceTimeUtc,
}

#[test]
fn should_serialize_and_deserialize_persistence_unit() {
    let event = CalendarEventDto {
        id: 1,
        title: "qqq".to_string(),
        start: OffsetDateTime::now_utc().into(),
        end: OffsetDateTime::now_utc().into(),
        icon: blinky_shared::calendar::CalendarEventIcon::Default,
        color: 0,
    };

    let persistence_unit = PersistenceUnit::new(PersistenceUnitKind::CalendarSyncInfo, &event);

    let result: CalendarEventDto = persistence_unit.deserialize().unwrap();

    assert_eq!(result, event);
}
