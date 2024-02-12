#![feature(vec_push_within_capacity)]

use std::ops::Add;

use blinky_shared::calendar::CalendarEvent;
use blinky_shared::events::Events;
use blinky_shared::{commands::Commands, modules::renderer::Renderer};
use display::SimDisplay;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X9, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle},
    text::Text,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use env_logger::{Builder, Target};
use log::{info, LevelFilter};
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};
use tokio::join;
use tokio::sync::broadcast;
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
        .worker_threads(4)
        .build()?;

    rt.block_on(async { main_async().await })?;
    Ok(())
}

async fn main_async() -> Result<(), Box<dyn std::error::Error>> {
    let (commands_sender, _) = broadcast::channel::<Commands>(100);
    let (events_sender, _) = broadcast::channel::<Events>(100);

    let commands_renderer = commands_sender.clone();
    let events_renderer = events_sender.clone();

    let renderer_task = tokio::spawn(async move {
        Renderer::<SimDisplay>::start(commands_renderer, events_renderer).await;
    });

    let commands_input = commands_sender.clone();

    tokio::task::spawn_blocking(move || {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        commands_input.send(Commands::StartDeepSleep).unwrap();
    });

    let startup_sequence = tokio::spawn(async move {
        let now_utc = OffsetDateTime::now_utc();
        let date = now_utc.date();

        let mut now = OffsetDateTime::new_utc(date, Time::MIDNIGHT);

        let start_time_utc = now + Duration::from_secs(3600 * 11);

        let duration = Duration::from_secs(3600 + 1800);

        commands_sender.send(Commands::ResumeRendering).unwrap();
        events_sender.send(Events::BatteryLevel(80)).unwrap();
        events_sender.send(Events::InSync(false)).unwrap();
        events_sender.send(Events::Temperature(20.0)).unwrap();

        events_sender
            .send(Events::CalendarEvent(CalendarEvent {
                id: 0,
                start: start_time_utc,
                end: start_time_utc + duration,
                title: "qqq".to_string(),
                icon: blinky_shared::calendar::CalendarEventIcon::Default,
                color: 0,
            }))
            .unwrap();

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

        loop {
            events_sender.send(Events::TimeNow(now)).unwrap();
            now = now.add(Duration::from_secs(1));
            sleep(Duration::from_millis(1000)).await;
        }
    });

    join!(renderer_task, startup_sequence);

    Ok(())
}
