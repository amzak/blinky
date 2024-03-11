use embedded_graphics::geometry::Point;

use crate::{calendar::CalendarEventIcon, display_interface::ClockDisplayInterface};
use embedded_icon::{
    mdi::{
        size12px as small_mdi_icons,
        size18px::{
            Battery10, Battery20, Battery30, Battery40, Battery50, Battery60, Battery70, Battery90,
            BatteryHigh, BatteryLow,
        },
    },
    NewIcon,
};

use super::graphics::Graphics;

#[macro_export]
macro_rules! icon {
    ($icon:ident, $size:ident, $color:ident) => {
        embedded_icon::mdi::size12px::$icon::new($color)
    };
}

pub fn render_event_icon<TDisplay: ClockDisplayInterface>(
    frame: &mut TDisplay::FrameBuffer<'_>,
    icon_type: CalendarEventIcon,
    center: Point,
    size: u8,
    color: TDisplay::ColorModel,
) {
    match icon_type {
        CalendarEventIcon::Meeting => {
            Graphics::<TDisplay>::icon_center(frame, center, &icon!(AccountMultiple, size, color))
        }
        CalendarEventIcon::Birthday => Graphics::<TDisplay>::icon_center(
            frame,
            center,
            &small_mdi_icons::CakeVariant::new(color),
        ),
        CalendarEventIcon::Trip => {
            Graphics::<TDisplay>::icon_center(frame, center, &small_mdi_icons::TrainCar::new(color))
        }
        CalendarEventIcon::Bus => {
            Graphics::<TDisplay>::icon_center(frame, center, &small_mdi_icons::Bus::new(color))
        }
        CalendarEventIcon::Train => Graphics::<TDisplay>::icon_center(
            frame,
            center,
            &small_mdi_icons::TrainVariant::new(color),
        ),
        CalendarEventIcon::Car => {
            Graphics::<TDisplay>::icon_center(frame, center, &small_mdi_icons::Car::new(color))
        }
        _ => {
            Graphics::<TDisplay>::icon_center(frame, center, &small_mdi_icons::Calendar::new(color))
        }
    }
}

pub fn render_battery_level_icon<TDisplay: ClockDisplayInterface>(
    frame: &mut TDisplay::FrameBuffer<'_>,
    battery_level: u16,
    center: Point,
    color: TDisplay::ColorModel,
) {
    match battery_level {
        91..=100 => Graphics::<TDisplay>::icon_center(frame, center, &BatteryHigh::new(color)),
        81..=90 => Graphics::<TDisplay>::icon_center(frame, center, &Battery90::new(color)),
        71..=80 => Graphics::<TDisplay>::icon_center(frame, center, &Battery70::new(color)),
        61..=70 => Graphics::<TDisplay>::icon_center(frame, center, &Battery60::new(color)),
        51..=60 => Graphics::<TDisplay>::icon_center(frame, center, &Battery50::new(color)),
        41..=50 => Graphics::<TDisplay>::icon_center(frame, center, &Battery40::new(color)),
        31..=40 => Graphics::<TDisplay>::icon_center(frame, center, &Battery30::new(color)),
        21..=30 => Graphics::<TDisplay>::icon_center(frame, center, &Battery20::new(color)),
        11..=20 => Graphics::<TDisplay>::icon_center(frame, center, &Battery10::new(color)),
        _ => Graphics::<TDisplay>::icon_center(frame, center, &BatteryLow::new(color)),
    }
}
