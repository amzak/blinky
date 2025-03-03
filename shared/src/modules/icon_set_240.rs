use embedded_graphics::prelude::RgbColor;
use embedded_icon::{mdi::size12px, EmbeddedIcon, Icon, NewIcon};

use super::icon_set::IconSet;

pub struct IconsSet240 {}

impl IconSet for IconsSet240 {
    fn get_bluetooth_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::BluetoothTransfer::new(color)
    }

    fn get_battery_level_100_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::BatteryHigh::new(color)
    }

    fn get_battery_level_90_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::Battery90::new(color)
    }

    fn get_battery_level_80_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::Battery80::new(color)
    }

    fn get_battery_level_70_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::Battery70::new(color)
    }

    fn get_battery_level_60_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::Battery60::new(color)
    }

    fn get_battery_level_50_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::Battery50::new(color)
    }

    fn get_battery_level_40_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::Battery40::new(color)
    }

    fn get_battery_level_30_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::Battery30::new(color)
    }

    fn get_battery_level_20_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::Battery20::new(color)
    }

    fn get_battery_level_10_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::BatteryLow::new(color)
    }

    fn get_meeting_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::AccountMultiple::new(color)
    }

    fn get_birthday_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::CakeVariant::new(color)
    }

    fn get_trip_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::TrainCar::new(color)
    }

    fn get_bus_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::Bus::new(color)
    }

    fn get_train_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::TrainVariant::new(color)
    }

    fn get_car_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::Car::new(color)
    }

    fn get_rain_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::WeatherRainy::new(color)
    }

    fn get_calendar_alert_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::CalendarAlert::new(color)
    }

    fn get_alarm_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::ClockAlert::new(color)
    }

    fn get_calendar_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size12px::Calendar::new(color)
    }
}
