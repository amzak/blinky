use esp_idf_hal::gpio::{Gpio21, Gpio36, Pins};

use super::mapping::PinsMapping;

pub struct TWatch2021Pins {
    adc_pin: Option<<Self as PinsMapping>::TAdcPin>,
    backlight_pin: <Self as PinsMapping>::TBacklightPin,
}

impl PinsMapping for TWatch2021Pins {
    type TAdcPin = Gpio36;
    type TBacklightPin = Gpio21;

    fn new(peripherals: Pins) -> TWatch2021Pins {
        TWatch2021Pins {
            adc_pin: Some(peripherals.gpio36),
            backlight_pin: peripherals.gpio21,
        }
    }

    fn get_adc_pin(&mut self) -> Self::TAdcPin {
        return self.adc_pin.take().unwrap();
    }

    fn get_backlight_pin(&mut self) -> &Self::TBacklightPin {
        return &self.backlight_pin;
    }

}
