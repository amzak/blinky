use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::pixelcolor::Rgb555;
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::text::Text;
use embedded_graphics_framebuf::FrameBuf;
use embedded_icon::mdi::size24px;
use enumflags2::BitFlags;
use time::{Duration, OffsetDateTime};

use time::macros::format_description;

use log::{debug, info};

use embedded_icon::mdi::size12px::{self};
use embedded_icon::prelude::*;
use tinytga::Tga;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use u8g2_fonts::U8g2TextStyle;

use crate::calendar::{self, CalendarEvent, CalendarEventKey};
use crate::calendar::{CalendarEventIcon, TimelyDataRecord};
use crate::commands::Commands;
use crate::display_interface::{ClockDisplayInterface, LayerType, RenderMode};
use crate::events::Events;
use crate::fasttrack::FastTrackRtcData;
use crate::message_bus::{BusHandler, BusSender, MessageBus};
use embedded_graphics::primitives::{PrimitiveStyle, StyledDrawable};
use embedded_graphics::{prelude::*, primitives};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::f32::consts::PI;
use std::marker::PhantomData;
use std::sync::Arc;

use super::fonts_set::FontSet;
use super::graphics::Graphics;
use super::icon_set::IconSet;
use super::relative::{RelativeCoordinate, RelativeSize};
use super::renderer_icons::render_battery_level_icon;
use super::renderer_icons::render_event_icon;

pub const HALF_DAY: Duration = Duration::hours(12);

pub struct Renderer<TDisplay, TFontSet: FontSet, TIconSet: IconSet> {
    _inner: PhantomData<TDisplay>,
    _font_set: PhantomData<TFontSet>,
    _icon_set: PhantomData<TIconSet>,
}

struct Context {
    tx: Sender<Events>,
    pause: bool,
}

#[derive(Debug)]
enum VisualMode {
    Normal,
    Details,
}

#[derive(Debug)]
struct ViewModel {
    is_past_first_frame: bool,
    is_charging: Option<bool>,
    battery_level: Option<u16>,
    ble_connected: Option<bool>,
    temperature: Option<i32>,
    calendar_events: BTreeSet<CalendarEvent>,
    timely_data: HashMap<i32, Vec<TimelyDataRecord>>,

    force_render_events: bool,

    time_vm: TimeViewModel,

    mode: VisualMode,

    alarm_counter: i16,
    gesture: u8,
}

#[derive(Debug)]
pub struct TimeViewModel {
    pub time: Option<OffsetDateTime>,
}

struct EventTagStyle<TColor> {
    icon: CalendarEventIcon,
    event_tag_size: RelativeSize,
    color: TColor,
    length: RelativeSize,
    thickness: RelativeSize,
}

impl<TColor> EventTagStyle<TColor> {
    fn default(icon: CalendarEventIcon, color: TColor) -> Self {
        Self {
            color,
            event_tag_size: 67u16.into(), //16,
            icon,
            length: 105u16.into(),  //25,
            thickness: 4u16.into(), //1
        }
    }

    fn large(icon: CalendarEventIcon, color: TColor) -> Self {
        Self {
            color,
            event_tag_size: 105u16.into(), //25,
            icon,
            length: 252u16.into(),  //60,
            thickness: 4u16.into(), //1
        }
    }
}

impl<TDisplay, TFontSet: FontSet, TIconSet: IconSet> BusHandler<Context>
    for Renderer<TDisplay, TFontSet, TIconSet>
{
    async fn event_handler(_bus: &BusSender, context: &mut Context, event: Events) {
        if context.pause && matches!(event, Events::TimeNow(_)) {
            return;
        }

        if !Self::is_renderable(&event) {
            return;
        }

        info!("sending event {:?} to render loop", event);
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

impl<TDisplay, TFontSet: FontSet, TIconSet: IconSet> Renderer<TDisplay, TFontSet, TIconSet> {
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
            | Events::Reminder(_)
            | Events::AccelerometerInterrupt(_)
            | Events::RtcAlarmInterrupt(_)
            | Events::Key1Press
            | Events::EventTimelyData(_) => {
                return true;
            }
            _ => false,
        }
    }
}

impl<TDisplay, TFontSet, TIconSet> Renderer<TDisplay, TFontSet, TIconSet>
where
    TDisplay: ClockDisplayInterface + 'static + Send,
    TFontSet: FontSet,
    TIconSet: IconSet,
{
    pub async fn start(bus: MessageBus, display: TDisplay, rtc_data: FastTrackRtcData) {
        info!("starting...");

        let (tx, rx) = channel::<Events>(16);

        let context = Context { tx, pause: false };

        let message_bus = bus.clone();
        let render_loop_task = tokio::task::spawn_blocking(move || {
            Self::render_loop(message_bus, rx, display, rtc_data);
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

        info!("rendering datetime {:?}", vm.time);

        let bounds = Self::render_time(frame, vm);
        Self::render_day(frame, vm, &bounds);
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

            Self::render_radial_line::<TDisplay::ColorModel>(
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
            U8g2TextStyle::new(TFontSet::get_clock_font(), TDisplay::ColorModel::WHITE);

        let time_as_text = vm.time.unwrap().format(&time_template).unwrap();

        let position = Self::get_center_point();

        Graphics::<TDisplay>::text_aligned(
            frame,
            &time_as_text,
            position.to_absolute(TDisplay::FRAME_BUFFER_SIDE),
            time_text_style,
            embedded_graphics::text::Alignment::Center,
        )
    }

    fn get_center_point() -> RelativeCoordinate {
        (1000 / 2, 1000 / 2).into()
    }

    fn render_day(
        frame: &mut TDisplay::FrameBuffer<'_>,
        vm: &TimeViewModel,
        time_text_bounds: &primitives::Rectangle,
    ) {
        let day_template = format_description!(version = 2, "[weekday repr:short]");

        let day_text_style = U8g2TextStyle::new(TFontSet::get_day_font(), RgbColor::WHITE);

        let day_as_text = vm.time.unwrap().format(&day_template).unwrap();

        let half_width = RelativeSize::from(time_text_bounds.size.width) / 2u32;

        let top_left =
            RelativeCoordinate::new(half_width + RelativeSize::from(500u16), 588u16.into());

        Graphics::<TDisplay>::text_aligned(
            frame,
            &day_as_text,
            top_left.to_absolute(TDisplay::FRAME_BUFFER_SIDE),
            day_text_style,
            embedded_graphics::text::Alignment::Right,
        );
    }

    fn render_radial_line<C>(
        frame: &mut TDisplay::FrameBuffer<'_>,
        angle: Angle,
        outer_radius: f32,
        length: f32,
        style: PrimitiveStyle<TDisplay::ColorModel>,
    ) -> Point {
        let radius_inner = outer_radius - length;

        let (sin, cos) = (Angle::from_radians(std::f32::consts::PI) - angle)
            .to_radians()
            .sin_cos();

        let zero_point = Self::get_center_point().to_absolute(TDisplay::FRAME_BUFFER_SIDE);

        let p1 = Point::new(
            (outer_radius * sin) as i32 + zero_point.x,
            (outer_radius * cos) as i32 + zero_point.y,
        );

        let p2 = Point::new(
            (radius_inner * sin) as i32 + zero_point.x,
            (radius_inner * cos) as i32 + zero_point.y,
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
            vm.temperature.unwrap(),
            char::from_u32(0x00b0).unwrap(),
        );

        let style_time = U8g2TextStyle::new(TFontSet::get_temperature_font(), RgbColor::WHITE);

        let point = Self::get_center_point() + (0, 84).into();

        Graphics::<TDisplay>::text_aligned(
            frame,
            &text,
            point.to_absolute(TDisplay::FRAME_BUFFER_SIDE),
            style_time,
            embedded_graphics::text::Alignment::Center,
        );
    }

    pub fn render_alarm(frame: &mut TDisplay::FrameBuffer<'_>, vm: &ViewModel) {
        let point = Self::get_center_point() + (0, 105).into();

        render_event_icon::<TDisplay, TIconSet>(
            frame,
            CalendarEventIcon::Alarm,
            point.to_absolute(TDisplay::FRAME_BUFFER_SIDE),
            12,
            RgbColor::RED,
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

        let text_style = U8g2TextStyle::new(TFontSet::get_temperature_font(), color);

        let text = format!(
            "{}'{}{}",
            duration.whole_minutes(),
            char::from_u32(0xe120 + 13).unwrap(),
            char::from_u32(0xe220 + 7).unwrap()
        );

        let top_left = Self::get_center_point() + (0, 84).into();

        Graphics::<TDisplay>::text_aligned(
            frame,
            &text,
            top_left.to_absolute(TDisplay::FRAME_BUFFER_SIDE),
            text_style,
            embedded_graphics::text::Alignment::Center,
        );
    }

    pub fn render_battery_level(frame: &mut TDisplay::FrameBuffer<'_>, vm: &ViewModel) {
        let point = RelativeCoordinate::from((504u16, 756u16)); //Point::new(120, 180);
        let absolute_point = point.to_absolute(TDisplay::FRAME_BUFFER_SIDE);

        if let Some(is_charging) = vm.is_charging {
            if is_charging {
                Graphics::<TDisplay>::icon_center(
                    frame,
                    absolute_point,
                    &size24px::BatteryCharging::new(TDisplay::ColorModel::WHITE),
                );

                return;
            }
        }

        if vm.battery_level.is_none() {
            return;
        }

        render_battery_level_icon::<TDisplay, TIconSet>(
            frame,
            vm.battery_level.unwrap(),
            absolute_point,
            TDisplay::ColorModel::WHITE,
        );
    }

    pub fn render_ble_connected(frame: &mut TDisplay::FrameBuffer<'_>, vm: &ViewModel) {
        if vm.ble_connected.is_none() {
            return;
        }

        let is_ble_connected = vm.ble_connected.unwrap();
        if is_ble_connected {
            let icon = TIconSet::get_bluetooth_icon(TDisplay::ColorModel::WHITE);

            let center = Self::get_center_point();
            let coord = center - (252, 0).into() + (0, 42).into();

            Graphics::<TDisplay>::icon(
                frame,
                coord.to_absolute(TDisplay::FRAME_BUFFER_SIDE),
                &icon,
            );
        };
    }

    fn render_loop(
        bus: MessageBus,
        mut rx: Receiver<Events>,
        display_param: TDisplay,
        rtc_data: FastTrackRtcData,
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
            force_render_events: false,
            mode: VisualMode::Normal,
            time_vm: TimeViewModel { time: rtc_data.now },
            is_past_first_frame: false,
            alarm_counter: if rtc_data.alarm_status { 10 } else { 0 },
            gesture: 0,
            timely_data: HashMap::new(),
        };

        let mut static_rendered = false;

        loop {
            info!("display loop waiting...");

            let event_opt = match rx.try_recv() {
                Ok(event) => Some(event),
                Err(err) => match err {
                    tokio::sync::mpsc::error::TryRecvError::Empty => {
                        debug!("render started...");

                        let mut render_layers_mask: BitFlags<LayerType> = LayerType::Clock.into();

                        if !static_rendered {
                            render_layers_mask |= LayerType::Static;
                            static_rendered = true;
                        }

                        if state.force_render_events {
                            render_layers_mask |= LayerType::Events;
                        }

                        Self::render(
                            &mut display,
                            &mut state,
                            render_layers_mask,
                            LayerType::Static | LayerType::Clock | LayerType::Events,
                        );

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
                if let Some(time) = view_model.time_vm.time {
                    if now.minute() != time.minute() {
                        view_model.force_render_events = true;
                    }
                }

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
            Events::Reminder(_reminder) => {
                view_model.alarm_counter = 10;
            }
            Events::AccelerometerInterrupt(gesture) => {
                view_model.gesture = gesture;
            }
            Events::EventTimelyData(data) => {
                append_timely_data(view_model, data);
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

        info!("rendering model: {:?}", vm);

        if render_layers_mask.contains(LayerType::Static) {
            display.render(LayerType::Static, RenderMode::Invalidate, |mut frame| {
                let tga: Tga<Rgb555> = if TDisplay::FRAME_BUFFER_SIDE <= 240 {
                    Tga::from_slice(include_bytes!(
                        "../../assets/blinky_watchface_magic_eye_240.tga"
                    ))
                    .unwrap()
                } else {
                    Tga::from_slice(include_bytes!(
                        "../../assets/blinky_watchface_magic_eye_466.tga"
                    ))
                    .unwrap()
                };

                let watchface = tga.pixels().filter_map(|x| {
                    if x.1 == Rgb555::BLACK {
                        None
                    } else {
                        let color = TDisplay::ColorModel::from(x.1);

                        let pixel = Pixel(x.0, color);
                        Some(pixel)
                    }
                });

                frame.draw_iter(watchface).unwrap();

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

                        if vm.alarm_counter > 0 {
                            Self::render_alarm(&mut frame, vm);
                            vm.alarm_counter -= 1;
                        } else if !Self::try_render_next_event_alert(&mut frame, vm) {
                            Self::render_temperature(&mut frame, vm);
                        }
                    }
                    VisualMode::Details => {
                        //Self::render_debug_info(&mut frame, vm);
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

        let mut text_style_underline = U8g2TextStyle::new(
            TFontSet::get_event_details_font(),
            TDisplay::ColorModel::WHITE,
        );

        text_style_underline
            .set_underline_color(embedded_graphics::text::DecorationColor::TextColor);

        let text_style_normal = U8g2TextStyle::new(
            TFontSet::get_event_details_font(),
            TDisplay::ColorModel::WHITE,
        );

        let zero_point = Self::get_center_point().to_absolute(TDisplay::FRAME_BUFFER_SIDE);

        let mut correction: i32 = 0;

        for (index, event) in current_events.enumerate() {
            Text::with_alignment(
                event.title.as_str(),
                zero_point + Point::new(0, ((index as i32 + correction as i32) * 2) * 12),
                &text_style_underline,
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
                Point::new(0, -20 + ((index as i32 + correction) * 2 + 1) * 12) + zero_point,
                &text_style_normal,
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

        if vm.force_render_events {
            frame.clear(TDisplay::ColorModel::default()).unwrap();
        }

        let now = vm.time_vm.time.unwrap();

        info!("rendering {} events...", vm.calendar_events.len());

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

        let current_finite_events = current_events
            .clone()
            .filter(|x| (x.end - now) < HALF_DAY && (x.end - x.start) < HALF_DAY);

        Self::render_current_finite_events(current_finite_events, frame, &now);

        let current_ambient_events: Vec<&CalendarEvent> = current_events
            .filter(|x| x.end - x.start >= HALF_DAY)
            .collect();

        Self::render_currrent_ambient_events(frame, current_ambient_events);

        let today_events = vm.calendar_events.iter().filter(|x| {
            (x.start > now) && (x.end - x.start) < HALF_DAY && (x.start - now) < HALF_DAY
        });

        Self::render_todays_events(today_events, frame, &now);

        vm.force_render_events = false;
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

        let radius: RelativeSize = 252.into();
        let angle = Angle::from_degrees(35.0 - 70.0 * index as f32);

        let (sin, cos) = (Angle::from_radians(std::f32::consts::PI) - angle)
            .to_radians()
            .sin_cos();

        let p1 = Point::new(
            (radius.to_absolute(TDisplay::FRAME_BUFFER_SIDE) as f32 * sin) as i32,
            (radius.to_absolute(TDisplay::FRAME_BUFFER_SIDE) as f32 * cos) as i32,
        );

        let color = if event.color == 0 {
            TDisplay::ColorModel::BLACK
        } else {
            TDisplay::ColorModel::from(RawU16::from_u32(event.color))
        };

        let style = EventTagStyle::large(event.icon, color);

        let zero_point = Self::get_center_point().to_absolute(TDisplay::FRAME_BUFFER_SIDE);

        Self::render_event_icon(frame, p1 + zero_point, &style)
    }

    fn render_current_finite_events<'a>(
        events: impl Iterator<Item = &'a CalendarEvent>,
        frame: &mut TDisplay::FrameBuffer<'_>,
        now: &OffsetDateTime,
    ) {
        for event in events {
            Self::render_todays_event(frame, &event, &now);
        }
    }

    fn render_todays_events<'a>(
        events: impl Iterator<Item = &'a CalendarEvent>,
        frame: &mut TDisplay::FrameBuffer<'_>,
        now: &OffsetDateTime,
    ) {
        for event in events {
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

        let visual_end = if event.end > now + HALF_DAY {
            now + HALF_DAY
        } else {
            event.end
        };

        let lane_line_thickness = RelativeSize::from(16);
        let lane_spacing = RelativeSize::from(117); //28;
        let outer_lane_diameter = RelativeSize::from(945); //225;
        let event_arc_diameter = outer_lane_diameter - (lane_spacing * event.lane as u16);

        Self::render_time_range_arc(
            frame,
            &now,
            &event.start,
            &visual_end,
            event_arc_diameter.to_absolute(TDisplay::FRAME_BUFFER_SIDE) as u32,
            lane_line_thickness.to_absolute(TDisplay::FRAME_BUFFER_SIDE) as u32,
            color,
        );

        let event_start_rel = event.start - now;

        let style = EventTagStyle::default(event.icon, TDisplay::ColorModel::BLACK);

        if event.start <= now {
            let zero_point = Self::get_center_point().to_absolute(TDisplay::FRAME_BUFFER_SIDE);
            let outer_radius: RelativeSize = event_arc_diameter / 2;
            let p1 = Point::new(0, outer_radius.to_absolute(TDisplay::FRAME_BUFFER_SIDE));
            Self::render_event_icon(frame, zero_point - p1, &style);
        } else {
            if event.start - now <= Duration::minutes(30) {}

            let start_angle = Angle::from_radians(
                (event_start_rel.whole_minutes() as f32 / HALF_DAY.whole_minutes() as f32)
                    * PI
                    * 2.0,
            );

            Self::render_event_tag(frame, start_angle, event_arc_diameter, style);
        };
    }

    fn render_time_range_arc(
        frame: &mut TDisplay::FrameBuffer<'_>,
        now: &OffsetDateTime,
        start: &OffsetDateTime,
        end: &OffsetDateTime,
        diameter: u32,
        thickness: u32,
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

        let style = PrimitiveStyle::with_stroke(color, thickness);

        let three_quaters = Angle::from_degrees(90.0);
        let start = start_angle - three_quaters;

        let top_left = Point::new(
            (TDisplay::FRAME_BUFFER_SIDE as i32 - diameter as i32) / 2,
            (TDisplay::FRAME_BUFFER_SIDE as i32 - diameter as i32) / 2,
        );

        debug!("arc from {:?} sweep {:?}", start, angle_sweep);

        if angle_sweep > Angle::from_degrees(1.0) {
            primitives::Arc::new(top_left, diameter, start, angle_sweep)
                .into_styled(style)
                .draw(frame)
                .unwrap();
        }
    }

    fn render_event_tag(
        frame: &mut TDisplay::FrameBuffer<'_>,
        angle: Angle,
        outer_diameter: RelativeSize,
        event_style: EventTagStyle<TDisplay::ColorModel>,
    ) {
        let outer_radius: RelativeSize = outer_diameter / 2;

        let event_style_rad: RelativeSize = event_style.event_tag_size / 2;
        let tag_rad_squared = event_style_rad.squared();

        let outer_rad_squared = outer_radius.squared();
        let arccos_arg: f32 = (outer_rad_squared as f32 * 2.0 - tag_rad_squared as f32)
            / (outer_rad_squared as f32 * 2.0);

        let angle_correction = arccos_arg.acos();

        let (sin, cos) = (Angle::from_radians(std::f32::consts::PI - angle_correction) - angle)
            .to_radians()
            .sin_cos();

        let zero_point = Self::get_center_point().to_absolute(TDisplay::FRAME_BUFFER_SIDE);

        let p1 = Point::new(
            (outer_radius.to_absolute(TDisplay::FRAME_BUFFER_SIDE) as f32 * sin) as i32
                + zero_point.x,
            (outer_radius.to_absolute(TDisplay::FRAME_BUFFER_SIDE) as f32 * cos) as i32
                + zero_point.y,
        );

        Self::render_event_icon(frame, p1, &event_style);
    }

    fn render_event_icon(
        frame: &mut TDisplay::FrameBuffer<'_>,
        point: Point,
        style: &EventTagStyle<TDisplay::ColorModel>,
    ) {
        let thickness = style.thickness;
        let event_tag_size = style.event_tag_size;
        let icon = style.icon;
        let color = style.color;

        let mut solid_style = PrimitiveStyle::with_stroke(
            TDisplay::ColorModel::WHITE,
            thickness.to_absolute_u32(TDisplay::FRAME_BUFFER_SIDE),
        );
        solid_style.fill_color = Some(TDisplay::ColorModel::WHITE);

        primitives::Circle::with_center(
            point,
            event_tag_size.to_absolute_u32(TDisplay::FRAME_BUFFER_SIDE),
        )
        .draw_styled(&solid_style, frame)
        .unwrap();

        render_event_icon::<TDisplay, TIconSet>(frame, icon, point, 6, color);
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

    fn render_debug_info(frame: &mut TDisplay::FrameBuffer<'_>, vm: &mut ViewModel) {
        let text = format!("gesture = {}", vm.gesture);

        let style_time = U8g2TextStyle::new(TFontSet::get_event_details_font(), RgbColor::WHITE);

        Graphics::<TDisplay>::text_aligned(
            frame,
            &text,
            Point::new(120, 140),
            style_time,
            embedded_graphics::text::Alignment::Center,
        );
    }
}

fn append_event(view_model: &mut ViewModel, calendar_event: &CalendarEvent) {
    let old_count = view_model.calendar_events.len();
    let updated = view_model.calendar_events.replace(calendar_event.clone());
    let new_count = view_model.calendar_events.len();

    if updated.is_some() || old_count != new_count {
        view_model.force_render_events = true;
    }
}

fn drop_events(view_model: &mut ViewModel, events_keys: Arc<Vec<CalendarEventKey>>) {
    let mut set: HashSet<CalendarEventKey> = HashSet::new();

    for event_key in events_keys.iter() {
        set.insert(event_key.clone());
    }

    view_model
        .calendar_events
        .retain(|x| !set.contains(&x.key()));

    if !set.is_empty() {
        info!("removed {} events", set.len());
        view_model.force_render_events = true;
    }
}

fn append_timely_data(view_model: &mut ViewModel, data: calendar::EventTimelyData) {
    let linked_event_id = &data.linked_event_id;

    view_model
        .timely_data
        .insert(*linked_event_id, data.timely_data);
}
