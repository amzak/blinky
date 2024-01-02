use esp_idf_hal::gpio::{AnyIOPin, Output, PinDriver};

pub struct Backlight<'a> {
    pin: PinDriver<'a, AnyIOPin, Output>,
}

impl Backlight<'_> {
    pub fn create(backlight_pin: i32) -> Self {
        let pin = unsafe { AnyIOPin::new(backlight_pin) };
        let mut pin_driver = PinDriver::output(pin).unwrap();

        pin_driver.set_low().unwrap();

        let backlight = Self { pin: pin_driver };
        backlight
    }

    pub fn on(&mut self) {
        self.pin.set_high().unwrap();
    }

    pub fn off(&mut self) {
        self.pin.set_low().unwrap();
    }
}
