use embedded_graphics::image::Image;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::pixelcolor::{Rgb555, Rgb565};
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::text::Text;
use embedded_graphics_framebuf::FrameBuf;
use enumflags2::BitFlags;
use time::{Duration, OffsetDateTime, Time};

use time::macros::format_description;

use log::{debug, info};

use embedded_icon::mdi::size12px::{self};
use embedded_icon::prelude::*;
use tinytga::Tga;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::calendar::CalendarEventIcon;
use crate::calendar::{CalendarEvent, CalendarEventKey};
use crate::commands::Commands;
use crate::display_interface::{ClockDisplayInterface, LayerType, RenderMode};
use crate::events::Events;
use crate::message_bus::{BusHandler, BusSender, MessageBus};
use embedded_graphics::primitives::{PrimitiveStyle, StyledDrawable};
use embedded_graphics::{mono_font::MonoTextStyle, prelude::*, primitives};
use std::collections::{BTreeSet, HashSet};
use std::f32::consts::PI;
use std::marker::PhantomData;
use std::sync::Arc;
use u8g2_fonts::{fonts, U8g2TextStyle};

use super::graphics::Graphics;
use super::renderer_icons::render_battery_level_icon;
use super::renderer_icons::render_event_icon;

pub const HALF_DAY: Duration = Duration::hours(12);

pub struct Renderer<TDisplay> {
    _inner: PhantomData<TDisplay>,
}

struct Context {
    tx: Sender<Events>,
    pause: bool,
}

enum VisualMode {
    Normal,
    Details,
}

struct ViewModel {
    is_past_first_frame: bool,
    is_charging: Option<bool>,
    battery_level: Option<u16>,
    ble_connected: Option<bool>,
    temperature: Option<f32>,
    calendar_events: BTreeSet<CalendarEvent>,

    should_update_events: bool,
    should_reset_events: bool,

    time_vm: TimeViewModel,

    mode: VisualMode,
}

pub struct TimeViewModel {
    pub time: Option<OffsetDateTime>,
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
            length: 60,
            thickness: 1,
        }
    }
}

impl<TDisplay> BusHandler<Context> for Renderer<TDisplay> {
    async fn event_handler(_bus: &BusSender, context: &mut Context, event: Events) {
        if context.pause && matches!(event, Events::TimeNow(_)) {
            return;
        }

        if !Self::is_renderable(&event) {
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

impl<TDisplay> Renderer<TDisplay> {
    fn is_renderable(event: &Events) -> bool {
        match event {
            Events::TimeNow(_)
            | Events::Temperature(_)
            | Events::BatteryLevel(_)
            | Events::Charging(_)
            | Events::BleClientConnected
            | Events::BleClientDisconnected
            | Events::CalendarEvent(_)
            | Events::CalendarEventsBatch(_)
            | Events::DropCalendarEventsBatch(_)
            | Events::Key1Press => {
                return true;
            }
            _ => false,
        }
    }
}

impl<TDisplay> Renderer<TDisplay>
where
    TDisplay: ClockDisplayInterface + 'static + Send,
{
    pub async fn start(bus: MessageBus, display: TDisplay, now: Option<OffsetDateTime>) {
        info!("starting...");

        let (tx, rx) = channel::<Events>(16);

        let context = Context { tx, pause: false };

        let message_bus = bus.clone();
        let render_loop_task = tokio::task::spawn_blocking(move || {
            Self::render_loop(message_bus, rx, display, now);
        });

        MessageBus::handle::<Context, Self>(bus, context).await;

        info!("waiting for render loop...");

        render_loop_task.await.unwrap();

        info!("done.");
    }

    pub fn render_datetime(frame: &mut TDisplay::FrameBuffer<'_>, vm: &TimeViewModel) {
        if vm.time.is_none() {
            return;
        }

        let bounds = Self::render_time(frame, vm);
        Self::render_day(frame, vm, &bounds);
        Self::draw_arrow(frame, vm);
    }

    fn render_clock_face(
        frame: &mut <TDisplay as ClockDisplayInterface>::FrameBuffer<'_>,
        vm: &mut ViewModel,
    ) {
        let width = 40;
        let half_width = width / 2;
        let value: RawU16 = Rgb565::CSS_DARK_SLATE_GRAY.into();
        let style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::from(value), width);
        let top_left = Point::new(half_width as i32, half_width as i32);

        Graphics::<TDisplay>::circle(
            frame,
            top_left,
            TDisplay::FRAME_BUFFER_SIDE as u32 - top_left.x as u32 * 2,
            style,
        );

        let top_left = Point::new(width as i32, width as i32);

        let width = 5;

        let value: RawU16 = Rgb565::CSS_LIGHT_SLATE_GRAY.into();
        let style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::from(value), width);

        Graphics::<TDisplay>::circle(
            frame,
            top_left,
            TDisplay::FRAME_BUFFER_SIDE as u32 - top_left.x as u32 * 2,
            style,
        );

        // let top_left = Point::new(2, 2);

        // let style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::YELLOW, 2);

        // let quater = Angle::from_degrees(90.0);

        // primitives::Arc::new(
        //     top_left,
        //     TDisplay::FRAME_BUFFER_SIDE as u32 - top_left.x as u32 * 2,
        //     Angle::from_radians(2.0 * std::f32::consts::PI * 5.0 / (60.0 * 12.0)) - quater,
        //     Angle::from_radians(2.0 * std::f32::consts::PI * 10.0 / (60.0 * 12.0)),
        // )
        // .into_styled(style)
        // .draw(frame)
        // .unwrap();

        // let style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::RED, 2);

        // primitives::Arc::new(
        //     top_left,
        //     TDisplay::FRAME_BUFFER_SIDE as u32 - top_left.x as u32 * 2,
        //     Angle::zero() - quater,
        //     Angle::from_radians(2.0 * std::f32::consts::PI * 5.0 / (60.0 * 12.0)),
        // )
        // .into_styled(style)
        // .draw(frame)
        // .unwrap();
    }

    fn render_clock_face_marks(
        frame: &mut <TDisplay as ClockDisplayInterface>::FrameBuffer<'_>,
        vm: &mut ViewModel,
    ) where
        TDisplay: ClockDisplayInterface,
    {
        const INTERVAL: Duration = Duration::minutes(30);
        const MAX_POS: Duration = HALF_DAY;

        let mut pos = Duration::ZERO;

        let radius: f32 = (TDisplay::FRAME_BUFFER_SIDE / 2) as f32;
        let length: f32 = 10.0;

        let mut style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::BLACK, 1);

        while pos < MAX_POS {
            style.stroke_width = 2;

            if pos == Duration::hours(0)
                || pos == Duration::hours(3)
                || pos == Duration::hours(6)
                || pos == Duration::hours(9)
            {
                style.stroke_width = 3;
            }

            let angle = (pos.whole_seconds() as f32 / MAX_POS.whole_seconds() as f32)
                * std::f32::consts::PI
                * 2.0;

            Self::draw_radial_line::<TDisplay::ColorModel>(
                frame,
                Angle::from_radians(angle),
                radius,
                length,
                style,
            );

            pos += INTERVAL;
        }
    }

    fn render_time(
        frame: &mut TDisplay::FrameBuffer<'_>,
        vm: &TimeViewModel,
    ) -> primitives::Rectangle {
        let time_template = format_description!(version = 2, "[hour repr:24]:[minute]:[second]");

        let time_text_style =
            U8g2TextStyle::new(fonts::u8g2_font_spleen16x32_mn, TDisplay::ColorModel::WHITE);

        let time_as_text = vm.time.unwrap().format(&time_template).unwrap();

        let position = Point::new(120, 120);

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
        vm: &TimeViewModel,
        time_text_bounds: &primitives::Rectangle,
    ) {
        let day_template = format_description!(version = 2, "[weekday repr:short]");

        let day_text_style = U8g2TextStyle::new(fonts::u8g2_font_wqy16_t_gb2312b, RgbColor::WHITE);

        let day_as_text = vm.time.unwrap().format(&day_template).unwrap();

        let half_width = time_text_bounds.size.width as i32 / 2;

        let top_left = Point::new(half_width + 119, 140);

        Graphics::<TDisplay>::text_aligned(
            frame,
            &day_as_text,
            top_left,
            day_text_style,
            embedded_graphics::text::Alignment::Right,
        );
    }

    fn draw_arrow(frame: &mut TDisplay::FrameBuffer<'_>, vm: &TimeViewModel) {
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

        let text = format!(
            "{}{}",
            char::from_u32(0xe0c0 + 14).unwrap(),
            vm.temperature.unwrap()
        );

        let style_time = U8g2TextStyle::new(fonts::u8g2_font_siji_t_6x10, RgbColor::WHITE);

        Graphics::<TDisplay>::text_aligned(
            frame,
            &text,
            Point::new(120, 140),
            style_time,
            embedded_graphics::text::Alignment::Center,
        );
    }

    pub fn try_render_next_event_alert(
        frame: &mut TDisplay::FrameBuffer<'_>,
        vm: &ViewModel,
    ) -> bool {
        let duration_to_next = Self::try_get_duration_to_next(vm);

        if duration_to_next.is_none() {
            return false;
        }

        let now = vm.time_vm.time.unwrap();

        Self::render_duration_to_next(frame, &now, duration_to_next.unwrap());

        return true;
    }

    fn try_get_duration_to_next(vm: &ViewModel) -> Option<Duration> {
        if vm.time_vm.time.is_none() {
            return None;
        }

        let scope_min = Duration::minutes(30);
        let mut half_a_day = Duration::hours(12);
        let ignore_sec = Duration::seconds(5);

        let mut duration_candidates = 0;

        let now = vm.time_vm.time.unwrap();

        for event in vm.calendar_events.iter() {
            if event.end - now > half_a_day {
                continue;
            }

            let till_start = event.start - now;

            if till_start > ignore_sec
                && till_start < scope_min
                && half_a_day > till_start
                && event.start > now
            {
                half_a_day = till_start;
                duration_candidates += 1;
            }
        }

        if duration_candidates == 0 {
            return None;
        }

        Some(half_a_day)
    }

    fn render_duration_to_next(
        frame: &mut TDisplay::FrameBuffer<'_>,
        now: &OffsetDateTime,
        duration: Duration,
    ) {
        let color = if duration < Duration::minutes(5) {
            RgbColor::RED
        } else {
            RgbColor::WHITE
        };

        let text_style = U8g2TextStyle::new(fonts::u8g2_font_siji_t_6x10, color);

        let text = format!(
            "{}'{}{}",
            duration.whole_minutes(),
            char::from_u32(0xe120 + 13).unwrap(),
            char::from_u32(0xe220 + 7).unwrap()
        );

        let half_width = TDisplay::FRAME_BUFFER_SIDE as i32 / 2;

        let top_left = Point::new(120, 140);

        Graphics::<TDisplay>::text_aligned(
            frame,
            &text,
            top_left,
            text_style,
            embedded_graphics::text::Alignment::Center,
        );
    }

    pub fn render_battery_level(frame: &mut TDisplay::FrameBuffer<'_>, vm: &ViewModel) {
        let point = Point::new(120, 180);

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
            let icon = size12px::BluetoothTransfer::new(TDisplay::ColorModel::WHITE);

            Graphics::<TDisplay>::icon(frame, Point::new(120 - 60, 130), &icon);
        };
    }

    fn render_loop(
        bus: MessageBus,
        mut rx: Receiver<Events>,
        display_param: TDisplay,
        now: Option<OffsetDateTime>,
    ) {
        info!("renderer loop started");

        let mut display = display_param; //TDisplay::create();

        info!("display initialized");

        let mut state: ViewModel = ViewModel {
            battery_level: None,
            is_charging: None,
            ble_connected: None,
            temperature: None,
            calendar_events: BTreeSet::new(),
            should_update_events: true,
            should_reset_events: false,
            mode: VisualMode::Normal,
            time_vm: TimeViewModel { time: now },
            is_past_first_frame: false,
        };

        let mut static_rendered = false;

        loop {
            info!("display loop waiting...");

            let event_opt = match rx.try_recv() {
                Ok(event) => Some(event),
                Err(err) => match err {
                    tokio::sync::mpsc::error::TryRecvError::Empty => {
                        debug!("render started...");

                        let all = LayerType::Static | LayerType::Clock | LayerType::Events;
                        let render_layers_mask = if static_rendered {
                            LayerType::Clock | LayerType::Events
                        } else {
                            static_rendered = true;
                            all
                        };

                        Self::render(&mut display, &mut state, render_layers_mask, all);

                        if !state.is_past_first_frame {
                            info!("first render");
                            state.is_past_first_frame = true;
                            bus.send_event(Events::FirstRender);
                        }

                        rx.blocking_recv()
                    }
                    tokio::sync::mpsc::error::TryRecvError::Disconnected => break,
                },
            };

            let event = event_opt.unwrap();

            if matches!(event, Events::Term) {
                break;
            }

            debug!("handling event {:?}", event);

            Self::try_apply_change(event, &mut state);
        }

        Self::render(
            &mut display,
            &mut state,
            LayerType::Static.into(),
            LayerType::Static.into(),
        );

        info!("renderer loop done");
    }

    fn try_apply_change(event: Events, view_model: &mut ViewModel) -> bool {
        let mut state_changed = true;

        match event {
            Events::TimeNow(now) => {
                view_model.time_vm.time = Some(now);
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
            Events::DropCalendarEventsBatch(batch) => {
                drop_events(view_model, batch);
            }
            Events::Key1Press => {
                view_model.mode = if matches!(view_model.mode, VisualMode::Normal) {
                    VisualMode::Details
                } else {
                    VisualMode::Normal
                }
            }
            _ => {
                state_changed = false;
            }
        }

        if let Some(now) = view_model.time_vm.time {
            view_model.calendar_events.retain(|x| x.end >= now);
        }

        return state_changed;
    }

    fn render(
        display: &mut TDisplay,
        vm: &mut ViewModel,
        render_layers_mask: BitFlags<LayerType>,
        merge_layers_mask: BitFlags<LayerType>,
    ) {
        debug!("render of {:?}", render_layers_mask);

        if render_layers_mask.contains(LayerType::Static) {
            display.render(LayerType::Static, RenderMode::Invalidate, |mut frame| {
                let watchface_data = include_bytes!("../../assets/blinky_watchface_magic_eye.tga");
                let tga: Tga<Rgb555> = Tga::from_slice(watchface_data).unwrap();

                let watchface = tga.pixels().filter_map(|x| {
                    if x.1 == Rgb555::BLACK {
                        None
                    } else {
                        let color = TDisplay::ColorModel::from(x.1);

                        let pixel = Pixel(x.0, color);
                        Some(pixel)
                    }
                });

                frame.draw_iter(watchface);

                frame
            });
        }

        if render_layers_mask.contains(LayerType::Clock) {
            display.render(LayerType::Clock, RenderMode::Invalidate, |mut frame| {
                match vm.mode {
                    VisualMode::Normal => {
                        Self::render_battery_level(&mut frame, vm);
                        Self::render_ble_connected(&mut frame, vm);
                        Self::render_datetime(&mut frame, &vm.time_vm);
                        if !Self::try_render_next_event_alert(&mut frame, vm) {
                            Self::render_temperature(&mut frame, vm);
                        }
                    }
                    VisualMode::Details => {
                        Self::render_current_events_details(&mut frame, vm);
                    }
                }

                frame
            });
        }

        if render_layers_mask.contains(LayerType::Events) {
            display.render(LayerType::Events, RenderMode::Ammend, |mut frame| {
                Self::render_events(&mut frame, vm);
                frame
            });
        }

        display.commit(merge_layers_mask);
    }

    fn render_current_events_details(frame: &mut TDisplay::FrameBuffer<'_>, vm: &mut ViewModel) {
        let now = vm.time_vm.time.unwrap();

        let current_events = vm
            .calendar_events
            .iter()
            .filter(|x| x.start <= now && x.end > now);

        let mut style_underline = MonoTextStyle::new(&FONT_6X10, TDisplay::ColorModel::WHITE);
        style_underline.set_underline_color(embedded_graphics::text::DecorationColor::TextColor);

        let style = MonoTextStyle::new(&FONT_6X10, TDisplay::ColorModel::WHITE);

        let center = Point::new_equal(TDisplay::FRAME_BUFFER_SIDE as i32 / 2);

        let mut correction: i32 = 0;

        for (index, event) in current_events.enumerate() {
            Text::with_alignment(
                event.title.as_str(),
                Point::new(0, -20 + ((index as i32 + correction) * 2) * 12) + center,
                style_underline,
                embedded_graphics::text::Alignment::Center,
            )
            .draw(frame)
            .unwrap();

            if event.description.is_empty() {
                correction -= 1;
                continue;
            }

            Text::with_alignment(
                event.description.as_str(),
                Point::new(0, -20 + ((index as i32 + correction) * 2 + 1) * 12) + center,
                style,
                embedded_graphics::text::Alignment::Center,
            )
            .draw(frame)
            .unwrap();
        }
    }

    fn render_events(frame: &mut TDisplay::FrameBuffer<'_>, vm: &mut ViewModel) {
        if vm.time_vm.time.is_none() {
            return;
        }

        if !(vm.should_update_events || vm.should_reset_events) {
            return;
        }

        if vm.should_reset_events {
            frame.clear(TDisplay::ColorModel::default()).unwrap();
        }

        let now = vm.time_vm.time.unwrap();

        debug!("rendering {} events...", vm.calendar_events.len());

        //const EVENT_TAG_SIZE: usize = 18;

        //let mut template_buf = [TDisplay::ColorModel::BLACK; EVENT_TAG_SIZE * EVENT_TAG_SIZE];
        //let mut fbuff = FrameBuf::new(&mut template_buf, EVENT_TAG_SIZE, EVENT_TAG_SIZE);

        // Self::draw_event_tag_template(
        //     &mut fbuff,
        //     Point::new(EVENT_TAG_SIZE as i32 / 2, EVENT_TAG_SIZE as i32 / 2),
        //     EVENT_TAG_SIZE as u32,
        // );

        let current_events = vm
            .calendar_events
            .iter()
            .filter(|x| x.start <= now && x.end > now);

        let current_finite_events = current_events.clone().filter(|x| x.end - now <= HALF_DAY);

        Self::render_current_finite_events(current_finite_events, frame, &now);

        let current_ambient_events: Vec<&CalendarEvent> =
            current_events.filter(|x| x.end - now > HALF_DAY).collect();

        Self::render_currrent_ambient_events(frame, current_ambient_events);

        let today_events = vm.calendar_events.iter().filter(|x| {
            (x.start > now) && (x.end - x.start) < HALF_DAY && (x.start - now) < HALF_DAY
        });

        Self::render_todays_events(today_events, frame, &now);

        vm.should_update_events = false;
    }

    fn render_currrent_ambient_events<'a>(
        frame: &mut TDisplay::FrameBuffer<'_>,
        events: Vec<&CalendarEvent>,
    ) {
        let count = events.len();

        for (index, event) in events.iter().enumerate() {
            Self::render_current_ambient_event(frame, event, index, count);
        }
    }

    fn render_current_ambient_event(
        frame: &mut TDisplay::FrameBuffer<'_>,
        event: &CalendarEvent,
        index: usize,
        count: usize,
    ) {
        if count > 2 {
            // ?
        }

        let radius = 60.0;
        let angle = Angle::from_degrees(35.0 - 70.0 * index as f32);

        let (sin, cos) = (Angle::from_radians(std::f32::consts::PI) - angle)
            .to_radians()
            .sin_cos();

        let p1 = Point::new(
            TDisplay::FRAME_BUFFER_SIDE as i32 / 2 + (radius * (sin)) as i32,
            TDisplay::FRAME_BUFFER_SIDE as i32 / 2 + (radius * (cos)) as i32,
        );

        let color = if event.color == 0 {
            TDisplay::ColorModel::WHITE
        } else {
            TDisplay::ColorModel::from(RawU16::from_u32(event.color))
        };

        let style = EventTagStyle::large(event.icon, color);

        Self::draw_event_tag(frame, p1, style)
    }

    fn render_current_finite_events<'a>(
        events: impl Iterator<Item = &'a CalendarEvent>,
        frame: &mut TDisplay::FrameBuffer<'_>,
        now: &OffsetDateTime,
    ) {
        for event in events {
            Self::render_current_finite_event(frame, &event, &now);
        }
    }

    fn render_current_finite_event(
        frame: &mut TDisplay::FrameBuffer<'_>,
        event: &CalendarEvent,
        now_ref: &OffsetDateTime,
    ) {
        let now = *now_ref;

        let color = if event.color == 0 {
            TDisplay::ColorModel::WHITE
        } else {
            TDisplay::ColorModel::from(RawU16::from_u32(event.color))
        };

        Self::render_time_range_arc(frame, &now, &event.start, &event.end, 238, 4, color);

        let event_start_rel = Duration::ZERO;

        let start_angle = Angle::from_radians(
            (event_start_rel.whole_minutes() as f32 / HALF_DAY.whole_minutes() as f32) * PI * 2.0,
        );

        let style = EventTagStyle::large(event.icon, color);

        Self::draw_event(frame, start_angle, style);
    }

    fn render_todays_events<'a>(
        events: impl Iterator<Item = &'a CalendarEvent>,
        frame: &mut TDisplay::FrameBuffer<'_>,
        now: &OffsetDateTime,
    ) {
        for event in events {
            let till_start = event.start - *now;

            Self::render_todays_event(frame, &event, &now);
        }
    }

    fn render_todays_event(
        frame: &mut TDisplay::FrameBuffer<'_>,
        event: &CalendarEvent,
        now_ref: &OffsetDateTime,
    ) {
        let now = *now_ref;

        let color = if event.color == 0 {
            TDisplay::ColorModel::WHITE
        } else {
            TDisplay::ColorModel::from(RawU16::from_u32(event.color))
        };

        Self::render_time_range_arc(frame, &now, &event.start, &event.end, 238, 4, color);

        let event_start_rel = event.start - now;

        let start_angle = Angle::from_radians(
            (event_start_rel.whole_minutes() as f32 / HALF_DAY.whole_minutes() as f32) * PI * 2.0,
        );

        let style = EventTagStyle::default(event.icon, color);

        Self::draw_event(frame, start_angle, style);
    }

    fn render_time_range_arc(
        frame: &mut TDisplay::FrameBuffer<'_>,
        now: &OffsetDateTime,
        start: &OffsetDateTime,
        end: &OffsetDateTime,
        diameter: u32,
        thickness: u8,
        color: TDisplay::ColorModel,
    ) {
        let event_start_rel = if start > now {
            *start - *now
        } else {
            Duration::ZERO
        };

        let event_end_rel = *end - *now;

        let start_angle = Angle::from_radians(
            (event_start_rel.whole_minutes() as f32 / HALF_DAY.whole_minutes() as f32) * PI * 2.0,
        );

        let end_angle = Angle::from_radians(
            (event_end_rel.whole_minutes() as f32 / HALF_DAY.whole_minutes() as f32) * PI * 2.0,
        );

        let angle_sweep = end_angle - start_angle;

        let style = PrimitiveStyle::with_stroke(color, thickness as u32);

        let three_quaters = Angle::from_degrees(90.0);
        let start = start_angle - three_quaters;

        let top_left = Point::new(
            (TDisplay::FRAME_BUFFER_SIDE as i32 - diameter as i32) / 2,
            (TDisplay::FRAME_BUFFER_SIDE as i32 - diameter as i32) / 2,
        );

        debug!("arc from {:?} sweep {:?}", start, angle_sweep);

        if angle_sweep > Angle::from_degrees(1.0) {
            primitives::Arc::new(
                top_left,
                diameter, //TDisplay::FRAME_BUFFER_SIDE as u32 - top_left.x as u32 * 2,
                start,
                angle_sweep,
            )
            .into_styled(style)
            .draw(frame)
            .unwrap();
        }
    }

    fn draw_event(
        frame: &mut TDisplay::FrameBuffer<'_>,
        angle: Angle,
        event_style: EventTagStyle<TDisplay::ColorModel>,
    ) {
        let initial_radius: f32 = TDisplay::FRAME_BUFFER_SIDE as f32 / 2.0;
        let length = event_style.length as f32;
        let thickness = event_style.thickness as u32;

        let style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::WHITE, thickness);

        let end_point = Self::draw_radial_line::<TDisplay::ColorModel>(
            frame,
            angle,
            initial_radius,
            length,
            style,
        );

        Self::draw_event_tag(frame, end_point, event_style);
    }

    fn draw_event_tag(
        frame: &mut TDisplay::FrameBuffer<'_>,
        point: Point,
        style: EventTagStyle<TDisplay::ColorModel>,
    ) {
        let thickness = style.thickness as u32;
        let event_tag_size = style.event_tag_size as u32;
        let icon = style.icon;
        let color = style.color;

        let mut solid_style = PrimitiveStyle::with_stroke(TDisplay::ColorModel::WHITE, thickness);
        solid_style.fill_color = Some(TDisplay::ColorModel::BLACK);

        primitives::Circle::with_center(point, event_tag_size)
            .draw_styled(&solid_style, frame)
            .unwrap();

        render_event_icon::<TDisplay>(frame, icon, point, 12, color);
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
        view_model.should_update_events = true;
    }
}

fn drop_events(view_model: &mut ViewModel, events_keys: Arc<Vec<CalendarEventKey>>) {
    let old_count = view_model.calendar_events.len();

    let mut set: HashSet<CalendarEventKey> = HashSet::new();

    for event_key in events_keys.iter() {
        set.insert(event_key.clone());
    }

    view_model
        .calendar_events
        .retain(|x| !set.contains(&x.key()));

    let new_count = view_model.calendar_events.len();

    if old_count != new_count {
        view_model.should_reset_events = true;
    }
}
