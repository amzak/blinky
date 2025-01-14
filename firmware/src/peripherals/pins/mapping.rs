use esp_idf_hal::gpio::{ADCPin, OutputPin, Pins};

pub trait PinsMapping {
    type TAdcPin: ADCPin;
    type TBacklightPin: OutputPin;

    fn new(peripherals: Pins) -> Self;

    fn get_adc_pin(&mut self) -> Self::TAdcPin;

    fn get_backlight_pin(&mut self) -> &Self::TBacklightPin;

}
