use embedded_graphics::prelude::RgbColor;
use embedded_icon::{mdi::size12px, EmbeddedIcon, Icon, NewIcon};

pub trait IconSet {
    fn get_bluetooth_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_battery_level_100_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_battery_level_90_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_battery_level_80_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_battery_level_70_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_battery_level_60_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_battery_level_50_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_battery_level_40_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_battery_level_30_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_battery_level_20_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_battery_level_10_icon<TColor: RgbColor>(
        color: TColor,
    ) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_meeting_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_birthday_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_trip_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_bus_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_train_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_car_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_rain_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_calendar_alert_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_alarm_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon>;

    fn get_calendar_icon<TColor: RgbColor>(color: TColor) -> Icon<TColor, impl EmbeddedIcon>;
}
