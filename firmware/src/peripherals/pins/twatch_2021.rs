use esp_idf_hal::gpio::{Gpio36, Pins};

use super::mapping::PinsMapping;

pub struct TWatch2021Pins {
    adc_pin: Option<Gpio36>,
}

impl PinsMapping for TWatch2021Pins {
    type TAdcPin = Gpio36;

    fn new(peripherals: Pins) -> TWatch2021Pins {
        TWatch2021Pins {
            adc_pin: Some(peripherals.gpio36),
        }
    }

    fn get_adc_pin(&mut self) -> Self::TAdcPin {
        return self.adc_pin.take().unwrap();
    }
}
