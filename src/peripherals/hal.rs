use embedded_hal::digital::v2::OutputPin;
use esp_idf_hal::gpio::{AnyIOPin, Output, OutputMode, Pin, PinDriver};
use esp_idf_hal::peripheral::Peripheral;
use crate::peripherals::backlight::Backlight;

pub struct HAL<'d> {
    pub backlight: Backlight<PinDriver<'d, AnyIOPin, Output>>
}

pub struct PinConfig {
    pub backlight: AnyIOPin
}

impl<'d> HAL<'d> {
    fn init_backlight(backlight_pin: AnyIOPin) -> Backlight<PinDriver<'d, AnyIOPin, Output>> {
        let pin_driver = PinDriver::output(backlight_pin).unwrap();
        Backlight::create(pin_driver)
    }

    pub fn new(config: PinConfig) -> HAL<'d> {
        let hal = Self {
            backlight: Self::init_backlight(config.backlight)
        };

        return hal;
    }

    pub fn init(&mut self) {
    }
}