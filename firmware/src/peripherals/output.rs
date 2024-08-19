use esp_idf_hal::gpio::{AnyIOPin, Output, PinDriver};

pub struct PinOutput<'a> {
    pin: PinDriver<'a, AnyIOPin, Output>,
}

impl PinOutput<'_> {
    pub fn create(pin_index: i32, initial_state: bool) -> Self {
        let pin = unsafe { AnyIOPin::new(pin_index) };
        let pin_driver = PinDriver::output(pin).unwrap();

        let mut pin_output = Self { pin: pin_driver };

        if initial_state {
            pin_output.on();
        } else {
            pin_output.off();
        }

        pin_output
    }

    pub fn on(&mut self) {
        self.pin.set_high().unwrap();
    }

    pub fn off(&mut self) {
        self.pin.set_low().unwrap();
    }
}
