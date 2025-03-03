#![feature(vec_push_within_capacity)]
#![feature(duration_constructors)]

use std::ops::Add;
use std::sync::Arc;

use blinky_shared::calendar::{CalendarEvent, CalendarEventKey, CalendarKind};
use blinky_shared::display_interface::ClockDisplayInterface;
use blinky_shared::events::Events;
use blinky_shared::fasttrack::FastTrackRtcData;
use blinky_shared::message_bus::MessageBus;
use blinky_shared::modules::fonts_set::FontSet466;
use blinky_shared::modules::icon_set_466::IconsSet466;
use blinky_shared::{commands::Commands, modules::renderer::Renderer};
use display::SimDisplay;
use env_logger::{Builder, Target};
use log::{info, LevelFilter};
use time::macros::datetime;
use time::{OffsetDateTime, Time, UtcOffset};
use tokio::join;
use tokio::time::{sleep, Duration};

mod display;

extern crate blinky_shared;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Builder::new()
        .target(Target::Stdout)
        .filter_level(LevelFilter::Debug)
        .init();
    info!("starting up");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .worker_threads(1)
        .build()?;

    rt.block_on(main_async())?;
    Ok(())
}

async fn main_async() -> Result<(), Box<dyn std::error::Error>> {
    let message_bus = MessageBus::new();

    let message_bus_clone = message_bus.clone();

    let display = SimDisplay::create();

    let rtc_data = FastTrackRtcData {
        alarm_status: false,
        now: None,
    };

    let renderer_task = Renderer::<SimDisplay, FontSet466, IconsSet466>::start(
        message_bus_clone,
        display,
        rtc_data,
    );

    let message_bus_clone = message_bus.clone();
    tokio::task::spawn_blocking(move || {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        message_bus_clone.send_cmd(Commands::StartDeepSleep);
    });

    let startup_sequence = async move {
        sleep(Duration::from_millis(1000)).await;

        let now_utc = datetime!(2000-01-01 12:59:59.5 UTC);
        let date = now_utc.date();

        let mut now = OffsetDateTime::new_in_offset(
            date,
            Time::from_hms(12, 0, 0).unwrap(),
            UtcOffset::from_hms(2, 0, 0).unwrap(),
        );

        let event_start_time = OffsetDateTime::new_utc(date, Time::MIDNIGHT)
            + Duration::from_hours(12)
            + Duration::from_hours(1)
            + Duration::from_secs(5);

        message_bus.send_event(Events::TimeNow(now));
        message_bus.send_cmd(Commands::ResumeRendering);
        message_bus.send_event(Events::BatteryLevel(80));
        message_bus.send_event(Events::BleClientConnected);
        message_bus.send_event(Events::Temperature(20));

        message_bus.send_event(Events::CalendarEvent(CalendarEvent {
            id: 0,
            kind: CalendarKind::Phone,
            start: now + Duration::from_secs(10),
            end: now + Duration::from_hours(1),
            title: "qqq1".to_string(),
            description: "some description".to_string(),
            icon: blinky_shared::calendar::CalendarEventIcon::Rain,
            color: 255,
            lane: 1,
        }));

        message_bus.send_event(Events::CalendarEvent(CalendarEvent {
            id: 1,
            kind: CalendarKind::Phone,
            start: now + Duration::from_hours(3),
            end: now + Duration::from_hours(5),
            title: "qqq1".to_string(),
            description: "some description".to_string(),
            icon: blinky_shared::calendar::CalendarEventIcon::Rain,
            color: 0,
            lane: 1,
        }));

        message_bus.send_event(Events::CalendarEvent(CalendarEvent {
            id: 2,
            kind: CalendarKind::Phone,
            start: now + Duration::from_hours(3),
            end: now + Duration::from_hours(5),
            title: "qqq2".to_string(),
            description: "some description".to_string(),
            icon: blinky_shared::calendar::CalendarEventIcon::CalendarAlert,
            color: 0,
            lane: 2,
        }));

        let sample_event = CalendarEvent {
            id: 4,
            kind: CalendarKind::Phone,
            start: event_start_time + Duration::from_mins(6),
            end: event_start_time + Duration::from_hours(11),
            title: "qqq3".to_string(),
            description: "description".to_string(),
            icon: blinky_shared::calendar::CalendarEventIcon::Car,
            color: 0,
            lane: 0,
        };

        message_bus.send_event(Events::CalendarEvent(sample_event.clone()));

        let mut toggler = false;

        loop {
            sleep(Duration::from_millis(1000)).await;

            message_bus.send_event(Events::TimeNow(now));

            toggler = !toggler;

            now = now.add(Duration::from_secs(1));
        }
    };

    let startup_sequence_task = tokio::spawn(startup_sequence);

    join!(renderer_task);

    startup_sequence_task.abort();

    Ok(())
}
