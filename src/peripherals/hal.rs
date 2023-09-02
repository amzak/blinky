use embedded_hal::digital::OutputPin;
use esp_idf_hal::gpio::{AnyIOPin, Output, OutputMode, Pin, PinDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::spi::SPI2;
use crate::peripherals::backlight::Backlight;
use crate::peripherals::display::ClockDisplay;

pub struct HAL<'d> {
    pub backlight: Backlight<PinDriver<'d, AnyIOPin, Output>>,
    pub display: ClockDisplay<'d>
}

pub struct PinConfig {
    pub backlight: AnyIOPin
}

impl<'d> HAL<'d> {
    fn init_backlight(backlight_pin: AnyIOPin) -> Backlight<PinDriver<'d, AnyIOPin, Output>> {
        let pin_driver = PinDriver::output(backlight_pin).unwrap();
        Backlight::create(pin_driver)
    }

    fn init_display() -> ClockDisplay<'d> {
        ClockDisplay::create()
    }

    pub fn new(config: PinConfig) -> HAL<'d> {
        let hal = Self {
            backlight: Self::init_backlight(config.backlight),
            display: Self::init_display()
        };

        return hal;
    }

    pub fn init(&mut self) {
    }
}