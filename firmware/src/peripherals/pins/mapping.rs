use esp_idf_hal::gpio::{ADCPin, Pins};

pub trait PinsMapping {
    type TAdcPin: ADCPin;

    fn new(peripherals: Pins) -> Self;

    fn get_adc_pin(&mut self) -> Self::TAdcPin;
}
