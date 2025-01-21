use esp_idf_hal::gpio::{
    Gpio0, Gpio10, Gpio11, Gpio12, Gpio13, Gpio14, Gpio15, Gpio17, Gpio4, Gpio6, Gpio7, PinDriver,
};

use super::mapping::PinsMapping;

pub struct TDisplay143 {
    adc_pin: Option<<Self as PinsMapping>::TAdcPin>,
    backlight_pin: <Self as PinsMapping>::TBacklightPin,
    spi_cs_pin: Option<<Self as PinsMapping>::TSpiCSPin>,
    spi_sclk_pin: Option<<Self as PinsMapping>::TSpiSCLK>,
    spi_sdo_pin: Option<<Self as PinsMapping>::TSpiSDO>,
    spi_dc_pin: Option<<Self as PinsMapping>::TSpiDC>,
    display_rst_pin: Option<<Self as PinsMapping>::TDisplayRst>,
    i2c_scl_pin: Option<<Self as PinsMapping>::TI2cScl>,
    i2c_sda_pin: Option<<Self as PinsMapping>::TI2cSda>,
}

impl PinsMapping for TDisplay143 {
    type TAdcPin = Gpio4;
    type TBacklightPin = Gpio0; // NOT USED

    type TSpiCSPin = Gpio10;

    type TSpiSCLK = Gpio12;

    type TSpiSDO = Gpio11;

    type TSpiSDO1 = Gpio13;
    type TSpiSDO2 = Gpio14;
    type TSpiSDO3 = Gpio15;

    type TSpiDC = PinDriver<'static, Gpio0, esp_idf_hal::gpio::Output>; // NOT USED

    type TDisplayRst = Gpio17;

    type TDisplayEn = Gpio16;

    type TI2cScl = Gpio6;

    type TI2cSda = Gpio7;

    fn new(peripherals: esp_idf_hal::gpio::Pins) -> Self {
        todo!()
    }

    fn get_adc_pin(&mut self) -> Self::TAdcPin {
        todo!()
    }

    fn get_backlight_pin(&mut self) -> &Self::TBacklightPin {
        todo!()
    }

    fn get_spi_cs_pin(&mut self) -> Self::TSpiCSPin {
        todo!()
    }

    fn get_spi_sclk_pin(&mut self) -> Self::TSpiSCLK {
        todo!()
    }

    fn get_spi_sdo_pin(&mut self) -> Self::TSpiSDO {
        todo!()
    }

    fn get_spi_dc_pin(&mut self) -> Self::TSpiDC {
        todo!()
    }

    fn get_display_rst_pin(&mut self) -> Self::TDisplayRst {
        todo!()
    }

    fn get_i2c_scl_pin(&mut self) -> Self::TI2cScl {
        todo!()
    }

    fn get_i2c_sda_pin(&mut self) -> Self::TI2cSda {
        todo!()
    }
}
