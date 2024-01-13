use time::{Duration, OffsetDateTime, Time};
use tokio::sync::broadcast::Sender;

use time::macros::format_description;

use log::info;

use embedded_icon::mdi::size18px::*;
use embedded_icon::prelude::*;

use crate::calendar::CalendarEvent;
use crate::commands::Commands;
use crate::display_interface::ClockDisplayInterface;
use crate::events::Events;
use embedded_graphics::primitives::{PrimitiveStyle, StyledDrawable};
use embedded_graphics::{mono_font::MonoTextStyle, prelude::*, primitives};
use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::mpsc::channel;
use u8g2_fonts::{fonts, U8g2TextStyle};

use super::graphics::Graphics;

pub struct Renderer<TDisplay> {
    _inner: PhantomData<TDisplay>,
}

pub struct ViewModel {
    is_charging: Option<bool>,
    battery_level: Option<u16>,
    sync_status: Option<bool>,
    temperature: Option<f32>,
    time: Option<OffsetDateTime>,
    calendar_events: HashSet<CalendarEvent>,
}

impl<TDisplay> Renderer<TDisplay>
where
    TDisplay: ClockDisplayInterface,
{
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

    fn render_datetime(frame: &mut TDisplay::FrameBuffer<'_>, vm: &ViewModel) {
        if vm.time.is_none() {
            return;
        }

        let bounds = Self::render_time(frame, vm);
        Self::render_day(frame, vm, &bounds);

        Self::draw_arrow(frame, vm);
    }

    fn render_time(frame: &mut TDisplay::FrameBuffer<'_>, vm: &ViewModel) -> primitives::Rectangle {
        let time_template = format_description!(version = 2, "[hour repr:24]:[minute]:[second]");

        let time_text_style =
            U8g2TextStyle::new(fonts::u8g2_font_spleen16x32_mn, TDisplay::ColorModel::WHITE);

        let time_as_text = vm.time.unwrap().format(&time_template).unwrap();

        let position = Point::new(120 - 10, 120);

        Graphics::<TDisplay>::text_aligned(
            frame,
            &time_as_text,
            position,
            time_text_style,
            embedded_graphics::text::Alignment::Center,
        )
    }

    fn render_day(
        frame: &mut TDisplay::FrameBuffer<'_>,
        vm: &ViewModel,
        time_text_bounds: &primitives::Rectangle,
    ) {
        let day_template = format_description!(version = 2, "[weekday repr:short]");

        let day_text_style = U8g2TextStyle::new(fonts::u8g2_font_wqy16_t_gb2312b, RgbColor::WHITE);

        let day_as_text = vm.time.unwrap().format(&day_template).unwrap();

        let half_width = time_text_bounds.size.width as i32 / 2;

        let top_left = Point::new(120 + half_width + 10, 120);

        Graphics::<TDisplay>::text_aligned(
            frame,
            &day_as_text,
            top_left,
            day_text_style,
            embedded_graphics::text::Alignment::Center,
        );
    }

    fn draw_arrow(frame: &mut TDisplay::FrameBuffer<'_>, vm: &ViewModel) {
        let now = vm.time.unwrap();

        let zero_time = Time::from_hms(0, 0, 0).unwrap();
        let today_midnight = now.replace_time(zero_time);
        let time_pos = now - today_midnight;

        let angle = ((time_pos.whole_minutes() as f32 / (12.0 * 60.0)) * 360.0) % 360.0;
        let length: f32 = 5.0;

        let radius = (TDisplay::FRAME_BUFFER_SIDE / 2) as f32;

        let radius_inner = radius - length;

        let (sin, cos) = (180.0 - angle).to_radians().sin_cos();

        let p1 = Point::new((radius * (sin + 1.0)) as i32, (radius * (cos + 1.0)) as i32);

        let p2 = Point::new(
            (radius_inner * sin + radius) as i32,
            (radius_inner * cos + radius) as i32,
        );

        let style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::RED, 3);

        primitives::Line::new(p1, p2)
            .draw_styled(&style, frame)
            .unwrap();
    }

    pub fn render_temperature(frame: &mut TDisplay::FrameBuffer<'_>, vm: &ViewModel) {
        if vm.temperature.is_none() {
            return;
        }

        let text = format!("{}{}", vm.temperature.unwrap(), char::from(176));

        let style_time = MonoTextStyle::new(
            &embedded_graphics::mono_font::iso_8859_3::FONT_8X13,
            TDisplay::ColorModel::WHITE,
        );

        Graphics::<TDisplay>::text_aligned(
            frame,
            &text,
            Point::new(120, 140),
            style_time,
            embedded_graphics::text::Alignment::Center,
        );
    }

    pub fn render_battery_level(frame: &mut TDisplay::FrameBuffer<'_>, vm: &ViewModel) {
        let point = Point::new(120 - 18 / 2, 215);

        if let Some(is_charging) = vm.is_charging {
            if is_charging {
                Graphics::<TDisplay>::icon(
                    frame,
                    point,
                    &BatteryCharging::new(TDisplay::ColorModel::WHITE),
                );
            }
        }

        if vm.battery_level.is_none() {
            return;
        }

        match vm.battery_level.unwrap() {
            91..=100 => Graphics::<TDisplay>::icon(
                frame,
                point,
                &BatteryHigh::new(TDisplay::ColorModel::WHITE),
            ),
            81..=90 => Graphics::<TDisplay>::icon(
                frame,
                point,
                &Battery90::new(TDisplay::ColorModel::WHITE),
            ),
            71..=80 => Graphics::<TDisplay>::icon(
                frame,
                point,
                &Battery70::new(TDisplay::ColorModel::WHITE),
            ),
            61..=70 => Graphics::<TDisplay>::icon(
                frame,
                point,
                &Battery60::new(TDisplay::ColorModel::WHITE),
            ),
            51..=60 => Graphics::<TDisplay>::icon(
                frame,
                point,
                &Battery50::new(TDisplay::ColorModel::WHITE),
            ),
            41..=50 => Graphics::<TDisplay>::icon(
                frame,
                point,
                &Battery40::new(TDisplay::ColorModel::WHITE),
            ),
            31..=40 => Graphics::<TDisplay>::icon(
                frame,
                point,
                &Battery30::new(TDisplay::ColorModel::WHITE),
            ),
            21..=30 => Graphics::<TDisplay>::icon(
                frame,
                point,
                &Battery20::new(TDisplay::ColorModel::WHITE),
            ),
            11..=20 => Graphics::<TDisplay>::icon(
                frame,
                point,
                &Battery10::new(TDisplay::ColorModel::WHITE),
            ),
            _ => Graphics::<TDisplay>::icon(
                frame,
                point,
                &BatteryLow::new(TDisplay::ColorModel::WHITE),
            ),
        }
    }

    pub fn render_sync_status(frame: &mut TDisplay::FrameBuffer<'_>, vm: &ViewModel) {
        if vm.sync_status.is_none() {
            return;
        }

        let color = if !vm.sync_status.unwrap() {
            TDisplay::ColorModel::WHITE
        } else {
            TDisplay::ColorModel::BLACK
        };

        let icon = Sync::new(color);

        Graphics::<TDisplay>::icon(frame, Point::new(120 - 18 / 2, 10), &icon);
    }

    fn render_loop(mut rx: std::sync::mpsc::Receiver<Events>) {
        let mut display = TDisplay::create();

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

    fn render_change(display: &mut TDisplay, event: Events, view_model: &mut ViewModel) {
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

    fn render(display: &mut TDisplay, vm: &mut ViewModel) {
        display.render(|mut frame| {
            let style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::WHITE, 1);
            let top_left = Point::new(3, 3);
            Graphics::<TDisplay>::circle(
                &mut frame,
                top_left,
                TDisplay::FRAME_BUFFER_SIDE as u32 - top_left.x as u32 * 2,
                style,
            );
            Self::render_battery_level(&mut frame, vm);
            Self::render_sync_status(&mut frame, vm);
            Self::render_temperature(&mut frame, vm);
            Self::render_datetime(&mut frame, vm);
            Self::render_events(&mut frame, vm);

            frame
        })
    }

    fn render_events(frame: &mut TDisplay::FrameBuffer<'_>, vm: &ViewModel) {
        if vm.time.is_none() {
            return;
        }

        let now = vm.time.unwrap();

        let zero_time = Time::from_hms(0, 0, 0).unwrap();

        let today_midnight = now.replace_time(zero_time);

        let style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::GREEN, 2);

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
                TDisplay::FRAME_BUFFER_SIDE as u32 - top_left.x as u32 * 2,
                Angle::from_degrees(start_angle + 270.0),
                Angle::from_degrees(angle_sweep),
            )
            .into_styled(style)
            .draw(frame)
            .unwrap();
        }
    }
}
