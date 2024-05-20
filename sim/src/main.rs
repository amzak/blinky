#![feature(vec_push_within_capacity)]
#![feature(duration_constructors)]

use std::ops::Add;
use std::sync::Arc;

use blinky_shared::calendar::{CalendarEvent, CalendarEventKey, CalendarKind};
use blinky_shared::events::Events;
use blinky_shared::message_bus::MessageBus;
use blinky_shared::{commands::Commands, modules::renderer::Renderer};
use display::SimDisplay;
use env_logger::{Builder, Target};
use log::{info, LevelFilter};
use time::{OffsetDateTime, Time};
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
        //.thread_stack_size(40 * 1024)
        .build()?;

    rt.block_on(main_async())?;
    Ok(())
}

async fn main_async() -> Result<(), Box<dyn std::error::Error>> {
    let message_bus = MessageBus::new();

    let message_bus_clone = message_bus.clone();

    let renderer_task = Renderer::<SimDisplay>::start(message_bus_clone);

    let message_bus_clone = message_bus.clone();
    tokio::task::spawn_blocking(move || {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        message_bus_clone.send_cmd(Commands::StartDeepSleep);
    });

    let startup_sequence = async move {
        sleep(Duration::from_millis(1000)).await;

        let now_utc = OffsetDateTime::now_utc();
        let date = now_utc.date();

        let mut now = OffsetDateTime::new_utc(date, Time::from_hms(12, 0, 0).unwrap()); //Time::MIDNIGHT);

        let event_start_time = OffsetDateTime::new_utc(date, Time::MIDNIGHT)
            + Duration::from_hours(12)
            + Duration::from_secs(5);

        let duration = Duration::from_hours(2);

        message_bus.send_event(Events::TimeNow(now));
        message_bus.send_cmd(Commands::ResumeRendering);
        message_bus.send_event(Events::BatteryLevel(80));
        message_bus.send_event(Events::BleClientConnected);
        message_bus.send_event(Events::Temperature(20.0));

        message_bus.send_event(Events::CalendarEvent(CalendarEvent {
            id: 0,
            kind: CalendarKind::Phone,
            start: event_start_time,
            end: event_start_time + duration,
            title: "qqq".to_string(),
            description: "some description".to_string(),
            icon: blinky_shared::calendar::CalendarEventIcon::Rain,
            color: 0,
        }));

        message_bus.send_event(Events::CalendarEvent(CalendarEvent {
            id: 1,
            kind: CalendarKind::Phone,
            start: now,
            end: now + Duration::from_days(1),
            title: "all day".to_string(),
            description: "".to_string(),
            icon: blinky_shared::calendar::CalendarEventIcon::Default,
            color: 0,
        }));

        let sample_event = CalendarEvent {
            id: 2,
            kind: CalendarKind::Phone,
            start: event_start_time + duration * 3,
            end: event_start_time + duration * 4,
            title: "qqq".to_string(),
            description: "description".to_string(),
            icon: blinky_shared::calendar::CalendarEventIcon::Car,
            color: 0,
        };

        //message_bus.send_event(Events::CalendarEvent(sample_event.clone()));

        /*
        for i in 1..40 {
            let offset: u64 = i * 60 * 20;

            events_sender
                .send(Events::CalendarEvent(CalendarEvent {
                    id: i as i64,
                    start: start_time_utc + Duration::from_secs(offset),
                    end: start_time_utc + Duration::from_secs(offset + 300),
                    title: "qqq".to_string(),
                    icon: blinky_shared::calendar::CalendarEventIcon::Default,
                    color: 0,
                }))
                .unwrap();
        }
         */

        let mut toggler = false;

        loop {
            sleep(Duration::from_millis(1000)).await;

            message_bus.send_event(Events::TimeNow(now));

            if (toggler) {
                message_bus.send_event(Events::CalendarEvent(sample_event.clone()));
            } else {
                let events = Arc::new(vec![sample_event.key()]);

                message_bus.send_event(Events::DropCalendarEventsBatch(events));

                message_bus.send_event(Events::Key1Press);
            }

            toggler = !toggler;

            now = now.add(Duration::from_secs(1));
        }
    };

    let startup_sequence_task = tokio::spawn(startup_sequence);

    join!(renderer_task);

    startup_sequence_task.abort();

    Ok(())
}
