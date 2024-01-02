use crate::peripherals::display::{ClockDisplay, FrameBuffer};
use crate::peripherals::hal::{Commands, Events};
use time::{Duration, OffsetDateTime, Time};
use tokio::sync::broadcast::Sender;

use time::macros::format_description;

use embedded_graphics::pixelcolor::Rgb565;
use log::info;
use profont::PROFONT_24_POINT;

use embedded_icon::mdi::size18px::*;
use embedded_icon::prelude::*;

use embedded_graphics::primitives::{PrimitiveStyle, StyledDrawable};
use embedded_graphics::text::Alignment;
use embedded_graphics::{
    image::Image,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    prelude::{DrawTarget, *},
    primitives,
    text::Text,
};
use std::collections::HashSet;
use std::sync::mpsc::channel;

use super::calendar_module::CalendarEvent;

pub struct Renderer {}

struct Graphics {}

pub struct ViewModel {
    is_charging: Option<bool>,
    battery_level: Option<u16>,
    sync_status: Option<bool>,
    temperature: Option<f32>,
    time: Option<OffsetDateTime>,
    calendar_events: HashSet<CalendarEvent>,
}

impl Renderer {
    pub async fn start(commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let mut pause = false;

        let (tx, rx) = channel::<Events>();

        let render_loop_task = tokio::task::spawn_blocking(move || {
            Self::render_loop(rx);
        });

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    match command {
                        Commands::PauseRendering => {
                            pause = true;
                        }
                        Commands::ResumeRendering => {
                            pause = false;
                        }
                        Commands::StartDeepSleep => {
                            tx.send(Events::Term).unwrap();
                            break;
                        }
                        _ => {}
                    }
                },
                Ok(event) = recv_event.recv() => {
                    if pause {
                        continue;
                    }

                    tx.send(event).unwrap();
                }
            }
        }

        render_loop_task.await.unwrap();

        info!("done");
    }

    pub fn render_time(frame: &mut FrameBuffer, vm: &ViewModel) {
        if vm.time.is_none() {
            return;
        }

        let template = format_description!(
            version = 2,
            "[weekday repr:short] [hour repr:24]:[minute]:[second]"
        );

        let text = vm.time.unwrap().format(&template).unwrap();
        let style_time = MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::WHITE);

        Graphics::text_aligned(
            frame,
            &text,
            Point::new(120, 120),
            style_time,
            embedded_graphics::text::Alignment::Center,
        );

        let now = vm.time.unwrap();

        let zero_time = Time::from_hms(0, 0, 0).unwrap();
        let today_midnight = now.replace_time(zero_time);
        let time_pos = now - today_midnight;

        let angle = ((time_pos.whole_minutes() as f32 / (12.0 * 60.0)) * 360.0) % 360.0;
        let length: f32 = 5.0;

        Self::draw_arrow(frame, angle, length);
    }

    fn draw_arrow(frame: &mut FrameBuffer, angle: f32, length: f32) {
        let radius = (ClockDisplay::FRAME_BUFFER_WIDTH / 2) as f32;

        let radius_inner = radius - length;

        let (sin, cos) = (180.0 - angle).to_radians().sin_cos();

        let p1 = Point::new((radius * (sin + 1.0)) as i32, (radius * (cos + 1.0)) as i32);

        let p2 = Point::new(
            (radius_inner * sin + radius) as i32,
            (radius_inner * cos + radius) as i32,
        );

        let style = PrimitiveStyle::with_stroke(Rgb565::RED, 3);

        primitives::Line::new(p1, p2)
            .draw_styled(&style, frame)
            .unwrap();
    }

    pub fn render_temperature(frame: &mut FrameBuffer, vm: &ViewModel) {
        if vm.temperature.is_none() {
            return;
        }

        let text = format!("{}{}", vm.temperature.unwrap(), char::from(176));

        let style_time = MonoTextStyle::new(
            &embedded_graphics::mono_font::iso_8859_3::FONT_8X13,
            Rgb565::WHITE,
        );

        Graphics::text_aligned(
            frame,
            &text,
            Point::new(120, 140),
            style_time,
            embedded_graphics::text::Alignment::Center,
        );
    }

    pub fn render_battery_level(frame: &mut FrameBuffer, vm: &ViewModel) {
        let point = Point::new(120 - 18 / 2, 220);

        if let Some(is_charging) = vm.is_charging {
            if is_charging {
                Graphics::icon(frame, point, &BatteryCharging::new(Rgb565::WHITE));
            }
        }

        if vm.battery_level.is_none() {
            return;
        }

        match vm.battery_level.unwrap() {
            91..=100 => Graphics::icon(frame, point, &BatteryHigh::new(Rgb565::WHITE)),
            81..=90 => Graphics::icon(frame, point, &Battery90::new(Rgb565::WHITE)),
            71..=80 => Graphics::icon(frame, point, &Battery70::new(Rgb565::WHITE)),
            61..=70 => Graphics::icon(frame, point, &Battery60::new(Rgb565::WHITE)),
            51..=60 => Graphics::icon(frame, point, &Battery50::new(Rgb565::WHITE)),
            41..=50 => Graphics::icon(frame, point, &Battery40::new(Rgb565::WHITE)),
            31..=40 => Graphics::icon(frame, point, &Battery30::new(Rgb565::WHITE)),
            21..=30 => Graphics::icon(frame, point, &Battery20::new(Rgb565::WHITE)),
            11..=20 => Graphics::icon(frame, point, &Battery10::new(Rgb565::WHITE)),
            _ => Graphics::icon(frame, point, &BatteryLow::new(Rgb565::WHITE)),
        }
    }

    pub fn render_sync_status(frame: &mut FrameBuffer, vm: &ViewModel) {
        if vm.sync_status.is_none() {
            return;
        }

        let color = if !vm.sync_status.unwrap() {
            Rgb565::WHITE
        } else {
            Rgb565::BLACK
        };

        let icon = Sync::new(color);

        Graphics::icon(frame, Point::new(120 - 18 / 2, 10), &icon);
    }

    fn render_loop(mut rx: std::sync::mpsc::Receiver<Events>) {
        let mut display = ClockDisplay::create();

        let mut state: ViewModel = ViewModel {
            battery_level: None,
            is_charging: None,
            sync_status: None,
            temperature: None,
            time: None,
            calendar_events: HashSet::new(),
        };

        loop {
            let event = rx.recv().unwrap();

            if matches!(event, Events::Term) {
                break;
            }

            Self::render_change(&mut display, event, &mut state)
        }
    }

    fn render_change(display: &mut ClockDisplay, event: Events, view_model: &mut ViewModel) {
        info!("{:?}", event);
        match event {
            Events::TimeNow(now) => {
                view_model.time = Some(now);
            }
            Events::Temperature(tmpr) => {
                view_model.temperature = Some(tmpr);
            }
            Events::BatteryLevel(level) => {
                view_model.battery_level = Some(level);
            }
            Events::Charging(is_charging) => {
                view_model.is_charging = Some(is_charging);
            }
            Events::InSync(status) => {
                view_model.sync_status = Some(status);
            }
            Events::CalendarEvent(calendar_event) => {
                view_model.calendar_events.replace(calendar_event);
            }
            _ => {}
        }

        if let Some(now) = view_model.time {
            view_model.calendar_events.retain(|x| x.end >= now);
        }

        Self::render(display, view_model);
    }

    fn render(display: &mut ClockDisplay, vm: &mut ViewModel) {
        display.render(|frame| {
            let style = PrimitiveStyle::with_stroke(Rgb565::WHITE, 1);
            let top_left = Point::new(3, 3);
            Graphics::circle(
                frame,
                top_left,
                ClockDisplay::FRAME_BUFFER_WIDTH as u32 - top_left.x as u32 * 2,
                style,
            );
            Self::render_battery_level(frame, vm);
            Self::render_sync_status(frame, vm);
            Self::render_temperature(frame, vm);
            Self::render_time(frame, vm);
            Self::render_events(frame, vm);
        })
    }

    fn render_events(frame: &mut FrameBuffer, vm: &ViewModel) {
        if vm.time.is_none() {
            return;
        }

        let now = vm.time.unwrap();

        let zero_time = Time::from_hms(0, 0, 0).unwrap();

        let today_midnight = now.replace_time(zero_time);

        let style = PrimitiveStyle::with_stroke(Rgb565::GREEN, 2);

        for event in vm.calendar_events.iter() {
            if event.end - event.start >= Duration::hours(12) {
                continue;
            }

            let event_start_rel = event.start - today_midnight;
            let event_end_rel = event.end - today_midnight;

            let start_angle =
                ((event_start_rel.whole_minutes() as f32 / (12.0 * 60.0)) * 360.0) % 360.0;
            let end_angle =
                ((event_end_rel.whole_minutes() as f32 / (12.0 * 60.0)) * 360.0) % 360.0;

            let angle_sweep = if event_start_rel == event_end_rel {
                2.0
            } else {
                end_angle - start_angle
            };

            info!(
                "event {} {} {} {} {}",
                event_start_rel, event_end_rel, start_angle, end_angle, angle_sweep
            );

            let top_left = Point::new(2, 2);
            primitives::Arc::new(
                top_left,
                ClockDisplay::FRAME_BUFFER_WIDTH as u32 - top_left.x as u32 * 2,
                Angle::from_degrees(start_angle + 270.0),
                Angle::from_degrees(angle_sweep),
            )
            .into_styled(style)
            .draw(frame)
            .unwrap();
        }
    }
}

impl Graphics {
    pub fn circle(
        frame: &mut FrameBuffer,
        coord: Point,
        diameter: u32,
        style: PrimitiveStyle<Rgb565>,
    ) {
        primitives::Circle::new(coord, diameter)
            .into_styled(style)
            .draw(frame)
            .unwrap();
    }

    pub fn icon(frame: &mut FrameBuffer, coord: Point, icon: &impl ImageDrawable<Color = Rgb565>) {
        Image::new(icon, coord).draw(frame).unwrap();
    }

    pub fn text_aligned(
        frame: &mut FrameBuffer,
        text: &str,
        coord: Point,
        style: MonoTextStyle<Rgb565>,
        alignment: Alignment,
    ) {
        let text = Text::with_alignment(text, coord, style, alignment);
        let bounding = text.bounding_box();

        let mut clipped = frame.clipped(&bounding);

        clipped.clear(Rgb565::BLACK).unwrap();

        text.draw(&mut clipped).unwrap();
    }

    pub fn text(frame: &mut FrameBuffer, text: &str, coord: Point) {
        let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

        Text::new(text, coord, style).draw(frame).unwrap();
    }
}
