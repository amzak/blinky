use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics_framebuf::FrameBuf;
use time::{Duration, OffsetDateTime, Time};

use time::macros::format_description;

use log::{debug, info};

use embedded_icon::mdi::{size12px, size18px};
use embedded_icon::prelude::*;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::calendar::CalendarEvent;
use crate::calendar::CalendarEventIcon;
use crate::commands::Commands;
use crate::display_interface::{ClockDisplayInterface, LayerType, RenderMode};
use crate::events::Events;
use crate::message_bus::{BusHandler, BusSender, MessageBus};
use embedded_graphics::primitives::{PrimitiveStyle, StyledDrawable};
use embedded_graphics::{mono_font::MonoTextStyle, prelude::*, primitives};
use std::borrow::Borrow;
use std::collections::{BTreeSet, HashSet};
use std::f32::consts::PI;
use std::marker::PhantomData;
use std::ops::Add;
use u8g2_fonts::{fonts, U8g2TextStyle};

use super::graphics::Graphics;
use super::renderer_icons::render_battery_level_icon;
use super::renderer_icons::render_event_icon;

pub struct Renderer<TDisplay> {
    _inner: PhantomData<TDisplay>,
}

struct Context {
    tx: Sender<Events>,
    pause: bool,
}

pub struct ViewModel {
    is_charging: Option<bool>,
    battery_level: Option<u16>,
    ble_connected: Option<bool>,
    temperature: Option<f32>,
    time: Option<OffsetDateTime>,
    calendar_events: BTreeSet<CalendarEvent>,

    force_update_events: bool,
}

struct EventTagStyle<TColor> {
    icon: CalendarEventIcon,
    event_tag_size: u8,
    color: TColor,
    length: u8,
    thickness: u8,
}

impl<TColor> EventTagStyle<TColor> {
    fn default(icon: CalendarEventIcon, color: TColor) -> Self {
        Self {
            color,
            event_tag_size: 18,
            icon,
            length: 25,
            thickness: 1,
        }
    }

    fn large(icon: CalendarEventIcon, color: TColor) -> Self {
        Self {
            color,
            event_tag_size: 25,
            icon,
            length: 35,
            thickness: 1,
        }
    }
}

impl<TDisplay> BusHandler<Context> for Renderer<TDisplay> {
    async fn event_handler(_bus: &BusSender, context: &mut Context, event: Events) {
        if context.pause && matches!(event, Events::TimeNow(_)) {
            return;
        }

        context.tx.send(event).await.unwrap();
    }

    async fn command_handler(_bus: &BusSender, context: &mut Context, command: Commands) {
        match command {
            Commands::PauseRendering => {
                context.pause = true;
            }
            Commands::ResumeRendering => {
                context.pause = false;
            }
            Commands::StartDeepSleep => {
                context.tx.send(Events::Term).await.unwrap();
            }
            _ => {}
        }
    }
}

impl<TDisplay> Renderer<TDisplay>
where
    TDisplay: ClockDisplayInterface,
{
    pub async fn start(bus: MessageBus) {
        info!("starting...");

        let (tx, rx) = channel::<Events>(16);

        let context = Context { tx, pause: false };

        let message_bus = bus.clone();
        let render_loop_task = tokio::task::spawn_blocking(|| {
            Self::render_loop(message_bus, rx);
        });

        MessageBus::handle::<Context, Self>(bus, context).await;

        info!("waiting for render loop...");

        render_loop_task.await.unwrap();

        info!("done.");
    }

    fn render_datetime(frame: &mut TDisplay::FrameBuffer<'_>, vm: &ViewModel) {
        if vm.time.is_none() {
            return;
        }

        let bounds = Self::render_time(frame, vm);
        Self::render_day(frame, vm, &bounds);
        Self::draw_arrow(frame, vm);
    }

    fn render_static(
        frame: &mut <TDisplay as ClockDisplayInterface>::FrameBuffer<'_>,
        vm: &mut ViewModel,
    ) {
        let style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::WHITE, 1);
        let top_left = Point::new(3, 3);
        Graphics::<TDisplay>::circle(
            frame,
            top_left,
            TDisplay::FRAME_BUFFER_SIDE as u32 - top_left.x as u32 * 2,
            style,
        );

        let top_left = Point::new(2, 2);

        let style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::YELLOW, 2);

        let quater = Angle::from_degrees(90.0);

        primitives::Arc::new(
            top_left,
            TDisplay::FRAME_BUFFER_SIDE as u32 - top_left.x as u32 * 2,
            Angle::from_radians(2.0 * std::f32::consts::PI * 5.0 / (60.0 * 12.0)) - quater,
            Angle::from_radians(2.0 * std::f32::consts::PI * 10.0 / (60.0 * 12.0)),
        )
        .into_styled(style)
        .draw(frame)
        .unwrap();

        let style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::RED, 2);

        primitives::Arc::new(
            top_left,
            TDisplay::FRAME_BUFFER_SIDE as u32 - top_left.x as u32 * 2,
            Angle::zero() - quater,
            Angle::from_radians(2.0 * std::f32::consts::PI * 5.0 / (60.0 * 12.0)),
        )
        .into_styled(style)
        .draw(frame)
        .unwrap();
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
        let point = Point::new(120, 205);

        if let Some(is_charging) = vm.is_charging {
            if is_charging {
                Graphics::<TDisplay>::icon_center(
                    frame,
                    point,
                    &size12px::BatteryCharging::new(TDisplay::ColorModel::WHITE),
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

    pub fn render_ble_connected(frame: &mut TDisplay::FrameBuffer<'_>, vm: &ViewModel) {
        if vm.ble_connected.is_none() {
            return;
        }

        let is_ble_connected = vm.ble_connected.unwrap();
        if is_ble_connected {
            let icon = size18px::Bluetooth::new(TDisplay::ColorModel::WHITE);

            Graphics::<TDisplay>::icon(frame, Point::new(120 - 18 / 2, 15), &icon);
        };
    }

    fn render_loop(_bus: MessageBus, mut rx: Receiver<Events>) {
        let mut display = TDisplay::create();
        let mut state: ViewModel = ViewModel {
            battery_level: None,
            is_charging: None,
            ble_connected: None,
            temperature: None,
            time: None,
            calendar_events: BTreeSet::new(),
            force_update_events: true,
        };

        let mut full_break = false;

        loop {
            loop {
                let event_opt = match rx.try_recv() {
                    Ok(event) => Some(event),
                    Err(err) => match err {
                        tokio::sync::mpsc::error::TryRecvError::Empty => {
                            Self::render(&mut display, &mut state);
                            rx.blocking_recv()
                        }
                        tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                            full_break = true;
                            None
                        }
                    },
                };

                if event_opt.is_none() {
                    full_break = true;
                    break;
                }

                let event = event_opt.unwrap();

                if matches!(event, Events::Term) {
                    full_break = true;
                    break;
                }

                Self::apply_change(event, &mut state);
            }

            if full_break {
                break;
            }
        }

        info!("renderer loop done");
    }

    fn apply_change(event: Events, view_model: &mut ViewModel) {
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
            Events::BleClientConnected => {
                view_model.ble_connected = Some(true);
            }
            Events::BleClientDisconnected => {
                view_model.ble_connected = Some(false);
            }
            Events::CalendarEvent(calendar_event) => {
                append_event(view_model, &calendar_event);
            }
            Events::CalendarEventsBatch(batch) => {
                let events = batch.iter();
                for item in events {
                    append_event(view_model, item);
                }
            }
            _ => {}
        }

        if let Some(now) = view_model.time {
            view_model.calendar_events.retain(|x| x.end >= now);
        }
    }

    fn render(display: &mut TDisplay, vm: &mut ViewModel) {
        display.render(LayerType::Static, RenderMode::Invalidate, |mut frame| {
            Self::render_static(&mut frame, vm);

            frame
        });

        display.render(LayerType::Clock, RenderMode::Invalidate, |mut frame| {
            Self::render_battery_level(&mut frame, vm);
            Self::render_ble_connected(&mut frame, vm);
            Self::render_datetime(&mut frame, vm);
            Self::render_temperature(&mut frame, vm);

            frame
        });

        display.render(LayerType::Events, RenderMode::Ammend, |mut frame| {
            Self::render_events(&mut frame, vm);
            frame
        });

        display.commit();
    }

    fn render_events(frame: &mut TDisplay::FrameBuffer<'_>, vm: &mut ViewModel) {
        if vm.time.is_none() {
            return;
        }

        if !vm.force_update_events {
            return;
        }

        let now = vm.time.unwrap();

        let half_day = Duration::hours(12);

        debug!("rendering {} events...", vm.calendar_events.len());

        const EVENT_TAG_SIZE: usize = 18;

        let mut template_buf = [TDisplay::ColorModel::BLACK; EVENT_TAG_SIZE * EVENT_TAG_SIZE];
        let mut fbuff = FrameBuf::new(&mut template_buf, EVENT_TAG_SIZE, EVENT_TAG_SIZE);

        // Self::draw_event_tag_template(
        //     &mut fbuff,
        //     Point::new(EVENT_TAG_SIZE as i32 / 2, EVENT_TAG_SIZE as i32 / 2),
        //     EVENT_TAG_SIZE as u32,
        // );

        for event in vm.calendar_events.iter() {
            if event.end - event.start >= half_day {
                continue;
            }

            if event.start - now > half_day {
                continue;
            }

            Self::render_event(frame, &event, &now, &template_buf);
        }

        vm.force_update_events = false;
    }

    fn render_event(
        frame: &mut TDisplay::FrameBuffer<'_>,
        event: &CalendarEvent,
        now_ref: &OffsetDateTime,
        template_buf: &[TDisplay::ColorModel; 18 * 18],
    ) {
        let now = *now_ref;

        let mut current_event = true;

        let mut event_start_rel = Duration::ZERO;

        if event.start > now {
            event_start_rel = event.start - now;
            current_event = false;
        }

        let half_a_day = Duration::hours(12);

        let event_end_rel = if event.end > now.add(half_a_day) {
            half_a_day
        } else {
            event.end - now
        };

        let start_angle = Angle::from_radians(
            (event_start_rel.whole_minutes() as f32 / half_a_day.whole_minutes() as f32) * PI * 2.0,
        );

        let end_angle = Angle::from_radians(
            (event_end_rel.whole_minutes() as f32 / half_a_day.whole_minutes() as f32) * PI * 2.0,
        );

        debug!(
            "event {} - {} {:?} - {:?} {:?} - {:?}",
            event.start, event.end, event_start_rel, event_end_rel, start_angle, end_angle
        );

        let angle_sweep = end_angle - start_angle;

        let color = if event.color == 0 {
            TDisplay::ColorModel::WHITE
        } else {
            TDisplay::ColorModel::from(RawU16::from_u32(event.color))
        };

        let style = PrimitiveStyle::with_stroke(color, 2);

        let three_quaters = Angle::from_degrees(90.0);
        let start = start_angle - three_quaters;

        let top_left = Point::new(2, 2);

        debug!("arc from {:?} sweep {:?}", start, angle_sweep);

        if angle_sweep > Angle::from_degrees(1.0) {
            primitives::Arc::new(
                top_left,
                TDisplay::FRAME_BUFFER_SIDE as u32 - top_left.x as u32 * 2,
                start,
                angle_sweep,
            )
            .into_styled(style)
            .draw(frame)
            .unwrap();
        }

        let style = if current_event {
            EventTagStyle::default(event.icon, color)
        } else {
            EventTagStyle::large(event.icon, color)
        };

        Self::draw_event(frame, start_angle, style);
    }

    fn draw_event(
        frame: &mut TDisplay::FrameBuffer<'_>,
        angle: Angle,
        style: EventTagStyle<TDisplay::ColorModel>,
    ) {
        let initial_radius: f32 = TDisplay::FRAME_BUFFER_SIDE as f32 / 2.0;
        let length = style.length as f32;
        let thickness = style.thickness as u32;
        let event_tag_size = style.event_tag_size as u32;
        let icon = style.icon;
        let color = style.color;

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

        primitives::Circle::with_center(end_point, event_tag_size)
            .draw_styled(&solid_style, frame)
            .unwrap();

        render_event_icon::<TDisplay>(frame, icon, end_point, 12, color);
    }

    fn draw_event_tag_template(
        frame: &mut FrameBuf<TDisplay::ColorModel, &mut [TDisplay::ColorModel; 18 * 18]>,
        center: Point,
        event_tag_size: u32,
    ) {
        let initial_radius: f32 = TDisplay::FRAME_BUFFER_SIDE as f32 / 2.0;
        let length = 25_f32;
        let thickness = 1;

        let mut solid_style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::WHITE, thickness);
        solid_style.fill_color = Some(TDisplay::ColorModel::BLACK);

        primitives::Circle::with_center(center, event_tag_size)
            .draw_styled(&solid_style, frame)
            .unwrap();

        Image::with_center(
            &size12px::Calendar::new(TDisplay::ColorModel::WHITE),
            center,
        )
        .draw(frame)
        .unwrap();
    }

    fn draw_event_from_template(
        frame: &mut <TDisplay as ClockDisplayInterface>::FrameBuffer<'_>,
        template_buf: &[<TDisplay as ClockDisplayInterface>::ColorModel; 18 * 18],
        center: Point,
        size: Size,
    ) {
        let iter = template_buf.into_iter().map(|x| *x);
        frame.fill_contiguous(
            &embedded_graphics::primitives::Rectangle::with_center(center, size),
            iter,
        );
    }
}

fn append_event(view_model: &mut ViewModel, calendar_event: &CalendarEvent) {
    let old_count = view_model.calendar_events.len();
    let updated = view_model.calendar_events.replace(calendar_event.clone());
    let new_count = view_model.calendar_events.len();

    if updated.is_some() || old_count != new_count {
        view_model.force_update_events = true;
    }
}
