use std::{pin::Pin, sync::Arc};

use blinky_shared::{
    calendar::{CalendarEvent, CalendarEventIcon, CalendarKind, TimelyDataRecord},
    events::Events,
    message_bus::MessageBus,
    modules::calendar_module::CalendarModule,
};
use time::{Date, Duration, OffsetDateTime, Time};

use crate::spy_module::SpyModule;

#[tokio::test]
async fn should_() {
    let message_bus = MessageBus::new();

    let message_bus_clone = message_bus.clone();
    let message_bus_clone2 = message_bus.clone();

    let calendar_module_task = CalendarModule::start(message_bus_clone);

    let startup_sequence = async move {
        let now = OffsetDateTime::new_utc(
            Date::from_calendar_date(2000, 1, 1),
            Time::from_hms(3, 0, 0),
        );

        let event = Events::TimeNow(now);
        message_bus.send_event(event);

        let timely_record = TimelyDataRecord {
            duration: Duration::seconds(60),
            linked_event_id: 1,
            start_at_hour: 2,
            data_marker: blinky_shared::calendar::TimelyDataMarker::Temperature,
            value: 0.0,
        };

        let event = Events::ReferenceTimelyDataBatch(Arc::new(vec![timely_record]));
        message_bus.send_event(event);

        let reference_calendar_events =
            Events::ReferenceCalendarEventUpdatesBatch(Arc::new(vec![CalendarEvent {
                kind: CalendarKind::Weather,
                id: 1,
                title: "Temperature".to_string(),
                start: now,
                end: now + Duration::days(1),
                icon: CalendarEventIcon::Temperature,
                color: 0,
                description: "".to_string(),
                lane: 0,
            }]));

        message_bus.send_event(reference_calendar_events);

        message_bus.send_event(Events::InSync(true));
    };

    let mut spy = SpyModule::new();

    let stop_event = Events::Temperature(0);
    let spy_task = spy.start(message_bus_clone2, stop_event);

    let tasks: Vec<Pin<Box<dyn futures::Future<Output = ()>>>> = vec![
        Box::pin(calendar_module_task),
        Box::pin(startup_sequence),
        Box::pin(spy_task),
    ];

    futures::future::join_all(tasks).await;
}
