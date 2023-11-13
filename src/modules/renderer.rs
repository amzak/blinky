use embedded_graphics::mono_font::MonoTextStyle;
use time::OffsetDateTime;
use crate::peripherals::hal::{Commands, Events};
use tokio::sync::broadcast::Sender;
use crate::peripherals::display::ClockDisplay;

use time::macros::format_description;

use embedded_graphics::{
    prelude::*,
};
use embedded_graphics::pixelcolor::Rgb565;
use log::info;
use profont::PROFONT_24_POINT;
use tokio::sync::Semaphore;

pub struct Renderer {

}

impl Renderer {
    pub async fn start(commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let mut display = ClockDisplay::create();

        let mut pause = false;

        let semaphore = Semaphore::new(1);

        display.clear();

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    let _s = semaphore.acquire().await.unwrap();
                    match command {
                        Commands::PauseRendering => {
                            pause = true;
                        }
                        Commands::ResumeRendering => {
                            pause = false;
                        }
                        Commands::StartDeepSleep => {
                            break;
                        }
                        _ => {}
                    }
                },
                Ok(event) = recv_event.recv() => {
                    if pause {
                        continue;
                    }
                    let _s = semaphore.acquire().await.unwrap();

                    match event {
                        Events::TimeNow(now) => {
                            Renderer::render_time(&mut display, now);
                        }
                        Events::Temperature(tmpr) => {
                            Renderer::render_temperature(&mut display, tmpr);
                        }
                        Events::BatteryLevel(level) => {
                            Renderer::render_battery_level(&mut display, level);
                        }
                        Events::Charging(is_charging) => {
                            Renderer::render_charging_status(&mut display, is_charging);
                        }
                        _ => {}
                    }
                }
            }
        }

        display.clear();

        info!("done");
    }

    pub fn render_time(display: &mut ClockDisplay, datetime: OffsetDateTime) {
        info!("render_time");

        let template = format_description!(
            version = 2,
            "[weekday repr:short] [hour repr:24]:[minute]:[second]"
        );

        let text = datetime.format(&template).unwrap();
        let style_time = MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::BLACK);

        display.text_aligned(&text, Point::new(120, 120), style_time, embedded_graphics::text::Alignment::Center);

        info!("render_time done");
    }

    pub fn render_temperature(display: &mut ClockDisplay, tmpr: f32) {
        let text = format!("{}{}", tmpr, char::from(176));

        let style_time = MonoTextStyle::new(&embedded_graphics::mono_font::iso_8859_3::FONT_8X13, Rgb565::BLACK);

        display.text_aligned(&text, Point::new(120, 140), style_time, embedded_graphics::text::Alignment::Center);
    }

    pub fn render_battery_level(display: &mut ClockDisplay, level: u16) {
        let text = format!("{}", level);

        let style_time = MonoTextStyle::new(&embedded_graphics::mono_font::iso_8859_1::FONT_8X13, Rgb565::BLACK);

        display.text_aligned(&text, Point::new(120, 160), style_time, embedded_graphics::text::Alignment::Center);
    }

    pub fn render_charging_status(display: &mut ClockDisplay, is_charging: bool) {
        let text = if is_charging { "*" } else { "" };

        let style_time = MonoTextStyle::new(&embedded_graphics::mono_font::iso_8859_1::FONT_8X13, Rgb565::BLACK);

        display.text_aligned(&text, Point::new(90, 160), style_time, embedded_graphics::text::Alignment::Center);
    }
}