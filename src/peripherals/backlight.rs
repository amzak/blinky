use esp_idf_hal::gpio::{AnyIOPin, Output, PinDriver};

pub struct Backlight<'a> {
    pin: PinDriver<'a, AnyIOPin, Output>,
    is_on: bool
}

impl Backlight<'_> {
    pub fn create(backlight_pin: i32) -> Self {
        let pin = unsafe { AnyIOPin::new(backlight_pin) };
        let pin_driver = PinDriver::output(pin).unwrap();

        let backlight = Self {
            pin: pin_driver,
            is_on: false
        };
        backlight
    }

    pub fn on(&mut self) {
        self.pin.set_high();
        self.is_on = true;
    }

    pub fn off(&mut self) {
        self.pin.set_low();
        self.is_on = false;
    }

    pub fn is_on(&self) -> bool {
        self.is_on
    }
}