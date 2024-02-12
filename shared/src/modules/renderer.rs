use embedded_graphics::pixelcolor::raw::RawU16;
use time::Instant;
use time::{Duration, OffsetDateTime, Time};
use tokio::runtime::{Builder, Handle};
use tokio::sync::broadcast::Sender;

use time::macros::format_description;

use log::{debug, info};

use embedded_icon::mdi::size18px::*;
use embedded_icon::prelude::*;
use tokio::sync::mpsc::{channel, Receiver};

use crate::calendar::CalendarEvent;
use crate::calendar::CalendarEventIcon;
use crate::commands::Commands;
use crate::display_interface::ClockDisplayInterface;
use crate::events::Events;
use embedded_graphics::primitives::{PrimitiveStyle, StyledDrawable};
use embedded_graphics::{mono_font::MonoTextStyle, prelude::*, primitives};
use std::collections::HashSet;
use std::marker::PhantomData;
use std::ops::Add;
use std::thread;
use u8g2_fonts::{fonts, U8g2TextStyle};

use super::graphics::Graphics;
use super::renderer_icons::render_battery_level_icon;
use super::renderer_icons::render_event_icon;

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

        let (tx, rx) = channel::<Events>(32);

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
                            tx.send(Events::Term).await.unwrap();
                            break;
                        }
                        _ => {}
                    }
                },
                Ok(event) = recv_event.recv() => {
                    if pause {
                        continue;
                    }

                    tx.send(event).await.unwrap();
                }
            }
        }

        info!("waiting for render loop...");

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

        let style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::WHITE, 1);
        let top_left = Point::new(3, 3);
        Graphics::<TDisplay>::circle(
            frame,
            top_left,
            TDisplay::FRAME_BUFFER_SIDE as u32 - top_left.x as u32 * 2,
            style,
        );
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
        let thickness = 3;

        let style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::RED, 3);

        Self::draw_radial_line::<TDisplay::ColorModel>(
            frame,
            Angle::from_degrees(angle),
            radius,
            length,
            style,
        );
    }

    fn draw_radial_line<C>(
        frame: &mut TDisplay::FrameBuffer<'_>,
        angle: Angle,
        initial_radius: f32,
        length: f32,
        style: PrimitiveStyle<TDisplay::ColorModel>,
    ) -> Point {
        let radius_inner = initial_radius - length;

        let (sin, cos) = (Angle::from_radians(std::f32::consts::PI) - angle)
            .to_radians()
            .sin_cos();

        let p1 = Point::new(
            (initial_radius * (sin + 1.0)) as i32,
            (initial_radius * (cos + 1.0)) as i32,
        );

        let p2 = Point::new(
            (radius_inner * sin + initial_radius) as i32,
            (radius_inner * cos + initial_radius) as i32,
        );

        primitives::Line::new(p1, p2)
            .draw_styled(&style, frame)
            .unwrap();

        p2
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
        let point = Point::new(120, 215);

        if let Some(is_charging) = vm.is_charging {
            if is_charging {
                Graphics::<TDisplay>::icon_center(
                    frame,
                    point,
                    &BatteryCharging::new(TDisplay::ColorModel::WHITE),
                );

                return;
            }
        }

        if vm.battery_level.is_none() {
            return;
        }

        render_battery_level_icon::<TDisplay>(
            frame,
            vm.battery_level.unwrap(),
            point,
            TDisplay::ColorModel::WHITE,
        );
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

    fn render_loop(mut rx: Receiver<Events>) {
        let mut display = TDisplay::create();

        let mut state: ViewModel = ViewModel {
            battery_level: None,
            is_charging: None,
            sync_status: None,
            temperature: None,
            time: None,
            calendar_events: HashSet::new(),
        };

        let handle = Handle::current();

        loop {
            let event_opt = handle.block_on(rx.recv());

            if event_opt.is_none() {
                break;
            }

            let event = event_opt.unwrap();

            if matches!(event, Events::Term) {
                break;
            }

            Self::render_change(&mut display, event, &mut state);
        }

        info!("renderer loop done");
    }

    fn render_change(display: &mut TDisplay, event: Events, view_model: &mut ViewModel) {
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
            let now = Instant::now();

            Self::render_battery_level(&mut frame, vm);

            let timing_battery = now.elapsed();

            Self::render_sync_status(&mut frame, vm);

            let timing_sync = now.elapsed();

            Self::render_temperature(&mut frame, vm);

            let timing_tmpr = now.elapsed();

            Self::render_datetime(&mut frame, vm);

            let timing_datetime = now.elapsed();

            Self::render_events(&mut frame, vm);

            let timing_events = now.elapsed();

            debug!(
                "rendering timing: battery {} sync {} tmpr {} datetime {} events {}",
                timing_battery, timing_sync, timing_tmpr, timing_datetime, timing_events
            );

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

        let half_day = Duration::hours(12);

        debug!("rendering {} events...", vm.calendar_events.len());

        for event in vm.calendar_events.iter() {
            if event.end - event.start >= half_day {
                continue;
            }

            if now - event.start > half_day {
                continue;
            }

            Self::render_event(frame, &event, &now);
        }
    }

    fn render_event(
        frame: &mut TDisplay::FrameBuffer<'_>,
        event: &CalendarEvent,
        now_ref: &OffsetDateTime,
    ) {
        let now = *now_ref;
        let event_start_rel = if event.start > now {
            event.start - now
        } else {
            Duration::ZERO
        };

        let half_a_day = Duration::hours(12);

        let event_end_rel = if event.end > now.add(half_a_day) {
            half_a_day
        } else {
            event.end - now
        };

        let start_angle = Angle::from_degrees(
            ((event_start_rel.whole_minutes() as f32 / (12.0 * 60.0)) * 360.0) % 360.0,
        );

        let end_angle = Angle::from_degrees(
            ((event_end_rel.whole_minutes() as f32 / (12.0 * 60.0)) * 360.0) % 360.0,
        );

        debug!(
            "event {} - {} {:?} - {:?}",
            event.start, event.end, start_angle, end_angle
        );

        let angle_sweep = if event_start_rel == event_end_rel {
            Angle::from_degrees(2.0)
        } else {
            end_angle - start_angle
        };

        let color = if event.color == 0 {
            TDisplay::ColorModel::WHITE
        } else {
            TDisplay::ColorModel::from(RawU16::from_u32(event.color))
        };

        let style = PrimitiveStyle::with_stroke(color, 2);

        let three_quaters = Angle::from_degrees(270.0);
        let start = start_angle + three_quaters;

        let top_left = Point::new(2, 2);

        debug!("arc from {:?} sweep {:?}", start, angle_sweep);

        primitives::Arc::new(
            top_left,
            TDisplay::FRAME_BUFFER_SIDE as u32 - top_left.x as u32 * 2,
            start,
            angle_sweep,
        )
        .into_styled(style)
        .draw(frame)
        .unwrap();

        Self::draw_event_tag(frame, start_angle, event.icon, color);
    }

    fn draw_event_tag(
        frame: &mut TDisplay::FrameBuffer<'_>,
        angle: Angle,
        icon: CalendarEventIcon,
        color: TDisplay::ColorModel,
    ) {
        let initial_radius: f32 = TDisplay::FRAME_BUFFER_SIDE as f32 / 2.0;
        let length = 25_f32;
        let thickness = 1;

        let style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::WHITE, thickness);

        let end_point = Self::draw_radial_line::<TDisplay::ColorModel>(
            frame,
            angle,
            initial_radius,
            length,
            style,
        );

        let mut solid_style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::WHITE, thickness);
        solid_style.fill_color = Some(TDisplay::ColorModel::BLACK);

        primitives::Circle::with_center(end_point, 18)
            .draw_styled(&solid_style, frame)
            .unwrap();

        render_event_icon::<TDisplay>(frame, icon, end_point, color);
    }
}
