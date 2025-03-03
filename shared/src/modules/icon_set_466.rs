use embedded_graphics::prelude::RgbColor;
use embedded_icon::{mdi::size24px, EmbeddedIcon, Icon, NewIcon};

use super::icon_set::IconSet;

pub struct IconsSet466 {}

impl IconSet for IconsSet466 {
    fn get_bluetooth_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::BluetoothTransfer::new(color)
    }

    fn get_battery_level_100_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::BatteryHigh::new(color)
    }

    fn get_battery_level_90_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::Battery90::new(color)
    }

    fn get_battery_level_80_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::Battery80::new(color)
    }

    fn get_battery_level_70_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::Battery70::new(color)
    }

    fn get_battery_level_60_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::Battery60::new(color)
    }

    fn get_battery_level_50_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::Battery50::new(color)
    }

    fn get_battery_level_40_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::Battery40::new(color)
    }

    fn get_battery_level_30_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::Battery30::new(color)
    }

    fn get_battery_level_20_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::Battery20::new(color)
    }

    fn get_battery_level_10_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::BatteryLow::new(color)
    }

    fn get_meeting_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::AccountMultiple::new(color)
    }

    fn get_birthday_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::CakeVariant::new(color)
    }

    fn get_trip_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::TrainCar::new(color)
    }

    fn get_bus_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::Bus::new(color)
    }

    fn get_train_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::TrainVariant::new(color)
    }

    fn get_car_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::Car::new(color)
    }

    fn get_rain_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::WeatherRainy::new(color)
    }

    fn get_calendar_alert_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::CalendarAlert::new(color)
    }

    fn get_alarm_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::ClockAlert::new(color)
    }

    fn get_calendar_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon> {
        size24px::Calendar::new(color)
    }
}
