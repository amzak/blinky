use blinky_shared::{
    calendar::{CalendarEventDto, CalendarKind},
    persistence::{PersistenceUnit, PersistenceUnitKind},
    reference_data::ReferenceTimeUtc,
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

#[tokio::test]
async fn should_serialize_and_deserialize_persistence_unit() {
    let event = CalendarEventDto {
        id: 1,
        title: "qqq".to_string(),
        start: OffsetDateTime::now_utc().into(),
        end: OffsetDateTime::now_utc().into(),
        icon: blinky_shared::calendar::CalendarEventIcon::Default,
        color: 0,
        kind: CalendarKind::Phone,
        description: "".to_string(),
        lane: 0,
    };

    let persistence_unit = PersistenceUnit::new(PersistenceUnitKind::CalendarSyncInfo, &event);

    let result: CalendarEventDto = persistence_unit.deserialize().await.unwrap();

    assert_eq!(result, event);
}
