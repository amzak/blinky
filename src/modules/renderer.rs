use embedded_graphics::mono_font::MonoTextStyle;
use time::OffsetDateTime;
use crate::peripherals::hal::{Commands, Events};
use tokio::sync::broadcast::{Sender, Receiver};
use crate::peripherals::display::ClockDisplay;

use time::macros::{datetime, format_description, offset};

use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_8X13},
        iso_8859_16::FONT_10X20,
    },
    prelude::{*, DrawTarget},
    text::Text,
};
use embedded_graphics::pixelcolor::Rgb565;

use tokio::time::{sleep, Duration};

pub struct Renderer {

}

impl Renderer {
    pub async fn start(commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let mut display = ClockDisplay::create();
        display.clear();

        loop {
            tokio::select! {
                Ok(event) = recv_event.recv() => {
                    match event {
                        Events::TimeNow(now) => {
                            Renderer::render_time(&mut display, now);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn render_time(display: &mut ClockDisplay, datetime: OffsetDateTime) {
        let template = format_description!(
            version = 2,
            "[hour repr:24]:[minute]:[second]"
        );

        let text = datetime.format(&template).unwrap();
        let style_time = MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK);

        display.text_aligned(&text, Point::new(120, 120), style_time, embedded_graphics::text::Alignment::Center);
    }

}