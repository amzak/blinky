use crate::{calendar::CalendarEventIcon, display_interface::ClockDisplayInterface};
use embedded_graphics::geometry::Point;

use super::{graphics::Graphics, icon_set::IconSet};

pub fn render_event_icon<TDisplay: ClockDisplayInterface, TIconSet: IconSet>(
    frame: &mut TDisplay::FrameBuffer<'_>,
    icon_type: CalendarEventIcon,
    center: Point,
    size: u8,
    color: TDisplay::ColorModel,
) {
    match icon_type {
        CalendarEventIcon::Meeting => {
            Graphics::<TDisplay>::icon_center(frame, center, &TIconSet::get_meeting_icon(color))
        }
        CalendarEventIcon::Birthday => {
            Graphics::<TDisplay>::icon_center(frame, center, &TIconSet::get_birthday_icon(color))
        }
        CalendarEventIcon::Trip => {
            Graphics::<TDisplay>::icon_center(frame, center, &TIconSet::get_trip_icon(color))
        }
        CalendarEventIcon::Bus => {
            Graphics::<TDisplay>::icon_center(frame, center, &TIconSet::get_bus_icon(color))
        }
        CalendarEventIcon::Train => {
            Graphics::<TDisplay>::icon_center(frame, center, &TIconSet::get_train_icon(color))
        }
        CalendarEventIcon::Car => {
            Graphics::<TDisplay>::icon_center(frame, center, &TIconSet::get_car_icon(color))
        }
        CalendarEventIcon::Rain => {
            Graphics::<TDisplay>::icon_center(frame, center, &TIconSet::get_rain_icon(color))
        }
        CalendarEventIcon::CalendarAlert => Graphics::<TDisplay>::icon_center(
            frame,
            center,
            &TIconSet::get_calendar_alert_icon(color),
        ),
        CalendarEventIcon::Alarm => {
            Graphics::<TDisplay>::icon_center(frame, center, &TIconSet::get_alarm_icon(color))
        }
        _ => Graphics::<TDisplay>::icon_center(frame, center, &TIconSet::get_calendar_icon(color)),
    }
}

pub fn render_battery_level_icon<TDisplay: ClockDisplayInterface, TIconSet: IconSet>(
    frame: &mut TDisplay::FrameBuffer<'_>,
    battery_level: u16,
    center: Point,
    color: TDisplay::ColorModel,
) {
    match battery_level {
        96..=100 => Graphics::<TDisplay>::icon_center(
            frame,
            center,
            &TIconSet::get_battery_level_100_icon(color),
        ),
        91..=95 => Graphics::<TDisplay>::icon_center(
            frame,
            center,
            &TIconSet::get_battery_level_90_icon(color),
        ),
        81..=90 => Graphics::<TDisplay>::icon_center(
            frame,
            center,
            &TIconSet::get_battery_level_80_icon(color),
        ),
        71..=80 => Graphics::<TDisplay>::icon_center(
            frame,
            center,
            &TIconSet::get_battery_level_70_icon(color),
        ),
        61..=70 => Graphics::<TDisplay>::icon_center(
            frame,
            center,
            &TIconSet::get_battery_level_60_icon(color),
        ),
        51..=60 => Graphics::<TDisplay>::icon_center(
            frame,
            center,
            &TIconSet::get_battery_level_50_icon(color),
        ),
        41..=50 => Graphics::<TDisplay>::icon_center(
            frame,
            center,
            &TIconSet::get_battery_level_40_icon(color),
        ),
        31..=40 => Graphics::<TDisplay>::icon_center(
            frame,
            center,
            &TIconSet::get_battery_level_30_icon(color),
        ),
        21..=30 => Graphics::<TDisplay>::icon_center(
            frame,
            center,
            &TIconSet::get_battery_level_20_icon(color),
        ),
        11..=20 => Graphics::<TDisplay>::icon_center(
            frame,
            center,
            &TIconSet::get_battery_level_10_icon(color),
        ),
        _ => Graphics::<TDisplay>::icon_center(
            frame,
            center,
            &TIconSet::get_battery_level_10_icon(color),
        ),
    }
}
