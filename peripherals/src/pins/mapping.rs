use std::{cell::RefCell, sync::Arc};

use critical_section::Mutex;
use esp_idf_hal::gpio::{ADCPin, IOPin, InputPin, OutputPin};

pub enum NotUsedPin {}

impl embedded_hal::digital::ErrorType for NotUsedPin {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::OutputPin for NotUsedPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

pub trait PinsMapping {
    type TAdcPin: ADCPin;
    type TBacklightPin: OutputPin;

    type TSpiCSPin: OutputPin;
    type TSpiSCLK: OutputPin;

    type TSpiSDO: IOPin;

    type TSpiSDO1: IOPin;
    type TSpiSDO2: IOPin;
    type TSpiSDO3: IOPin;

    type TSpiDC: embedded_hal::digital::OutputPin;

    type TDisplayEn: OutputPin;
    type TDisplayRst: embedded_hal::digital::OutputPin;

    type TI2cScl: IOPin;
    type TI2cSda: IOPin;

    type TTouchInterrupt: InputPin;
    type TButton1: InputPin;

    fn get_adc_pin(&mut self) -> Self::TAdcPin;

    fn get_backlight_pin(&mut self) -> Option<Self::TBacklightPin>;

    fn get_spi_cs_pin(&mut self) -> Self::TSpiCSPin;

    fn get_spi_sclk_pin(&mut self) -> Self::TSpiSCLK;

    fn get_spi_sdo_pin(&mut self) -> Self::TSpiSDO;

    fn get_spi_dc_pin(&mut self) -> Self::TSpiDC;

    fn get_display_rst_pin(&mut self) -> Self::TDisplayRst;

    fn get_i2c_scl_pin(&mut self) -> Self::TI2cScl;

    fn get_i2c_sda_pin(&mut self) -> Self::TI2cSda;

    fn get_spi_sdo1_pin(&mut self) -> Self::TSpiSDO1;

    fn get_spi_sdo2_pin(&mut self) -> Self::TSpiSDO2;

    fn get_spi_sdo3_pin(&mut self) -> Self::TSpiSDO3;

    fn get_display_en_pin(&mut self) -> Self::TDisplayEn;

    fn get_touch_int_pin(&mut self) -> Self::TTouchInterrupt;

    fn get_touch_int_pin_index(&self) -> i32;

    fn get_button1_pin(&mut self) -> Self::TButton1;

    fn get_button1_pin_index(&self) -> i32;
}

pub struct I2cProxyAsync<T> {
    bus: Arc<Mutex<RefCell<T>>>,
}
