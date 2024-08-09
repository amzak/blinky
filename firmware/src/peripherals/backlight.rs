use esp_idf_hal::gpio::{AnyIOPin, Output, PinDriver};
use log::info;

pub struct Backlight<'a> {
    pin: PinDriver<'a, AnyIOPin, Output>,
}

impl Backlight<'_> {
    pub fn create(backlight_pin: i32, initial_state: bool) -> Self {
        let pin = unsafe { AnyIOPin::new(backlight_pin) };
        let pin_driver = PinDriver::output(pin).unwrap();

        let mut backlight = Self { pin: pin_driver };

        if initial_state {
            backlight.on();
        } else {
            backlight.off();
        }

        backlight
    }

    pub fn on(&mut self) {
        self.pin.set_high().unwrap();
        info!("backlight on");
    }

    pub fn off(&mut self) {
        self.pin.set_low().unwrap();
        info!("backlight off");
    }
}
