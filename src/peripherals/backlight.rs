use embedded_hal::digital::OutputPin;
use esp_idf_hal::gpio::{AnyIOPin, Output, Pin, PinDriver};

pub struct Backlight<P> {
    pin: P
}

impl<P> Backlight<P> where P: OutputPin {
    pub fn create(pin_driver: P) -> Self {
        let backlight = Self {
            pin: pin_driver
        };
        backlight
    }

    pub fn on(&mut self) {
        self.pin.set_high();
    }

    pub fn off(&mut self) {
        self.pin.set_low();
    }
}