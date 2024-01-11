#![feature(vec_push_within_capacity)]

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
use time::OffsetDateTime;
use tokio::join;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

mod display;

extern crate blinky_shared;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .worker_threads(2)
        //.thread_stack_size(30 * 1024)
        .build()?;

    rt.block_on(async { main_async().await })?;
    Ok(())
}

async fn main_async() -> Result<(), Box<dyn std::error::Error>> {
    let (commands_sender, _) = broadcast::channel::<Commands>(32);
    let (events_sender, _) = broadcast::channel::<Events>(32);

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
        commands_sender.send(Commands::ResumeRendering).unwrap();
        events_sender.send(Events::BatteryLevel(80)).unwrap();
        events_sender.send(Events::InSync(false)).unwrap();
        events_sender.send(Events::Temperature(20.0)).unwrap();

        loop {
            let time_utc = OffsetDateTime::now_utc();
            events_sender.send(Events::TimeNow(time_utc)).unwrap();
            sleep(Duration::from_millis(1000)).await;
        }
    });

    join!(renderer_task, startup_sequence);

    Ok(())
}
