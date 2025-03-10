use esp_idf_hal::gpio::{
    Gpio0, Gpio13, Gpio14, Gpio15, Gpio19, Gpio21, Gpio25, Gpio26, Gpio27, Gpio36, PinDriver, Pins,
};

use super::mapping::PinsMapping;

pub struct TWatch2021Pins {
    adc_pin: Option<<Self as PinsMapping>::TAdcPin>,
    backlight_pin: <Self as PinsMapping>::TBacklightPin,
    spi_cs_pin: Option<<Self as PinsMapping>::TSpiCSPin>,
    spi_sclk_pin: Option<<Self as PinsMapping>::TSpiSCLK>,
    spi_sdo_pin: Option<<Self as PinsMapping>::TSpiSDO>,
    spi_dc_pin: Option<<Self as PinsMapping>::TSpiDC>,
    display_rst_pin: Option<<Self as PinsMapping>::TDisplayRst>,
    i2c_scl_pin: Option<<Self as PinsMapping>::TI2cScl>,
    i2c_sda_pin: Option<<Self as PinsMapping>::TI2cSda>,

    touch_int: Option<<Self as PinsMapping>::TTouchInterrupt>,
    button1: Option<<Self as PinsMapping>::TButton1>,
}

impl PinsMapping for TWatch2021Pins {
    type TAdcPin = Gpio36;
    type TBacklightPin = Gpio21;

    type TSpiCSPin = Gpio15;
    type TSpiSCLK = Gpio14;
    type TSpiSDO = Gpio13;
    type TSpiDC = PinDriver<'static, Gpio19, esp_idf_hal::gpio::Output>;

    type TDisplayRst = PinDriver<'static, Gpio27, esp_idf_hal::gpio::Output>;

    type TI2cScl = Gpio25;
    type TI2cSda = Gpio26;

    type TTouchInterrupt = Gpio32;
    type TButton1 = Gpio34;

    fn new(peripherals: Pins) -> TWatch2021Pins {
        TWatch2021Pins {
            adc_pin: Some(peripherals.gpio36),
            backlight_pin: peripherals.gpio21,
            spi_cs_pin: Some(peripherals.gpio15),
            spi_sclk_pin: Some(peripherals.gpio14),
            spi_sdo_pin: Some(peripherals.gpio13),
            spi_dc_pin: Some(PinDriver::output(peripherals.gpio19).unwrap()),
            display_rst_pin: Some(PinDriver::output(peripherals.gpio27).unwrap()),
            i2c_scl_pin: Some(peripherals.gpio25),
            i2c_sda_pin: Some(peripherals.gpio26),
        }
    }

    fn get_adc_pin(&mut self) -> Self::TAdcPin {
        return self.adc_pin.take().unwrap();
    }

    fn get_backlight_pin(&mut self) -> &Self::TBacklightPin {
        return &self.backlight_pin;
    }

    fn get_spi_cs_pin(&mut self) -> Self::TSpiCSPin {
        return self.spi_cs_pin.take().unwrap();
    }

    fn get_spi_sclk_pin(&mut self) -> Self::TSpiSCLK {
        return self.spi_sclk_pin.take().unwrap();
    }

    fn get_spi_sdo_pin(&mut self) -> Self::TSpiSDO {
        return self.spi_sdo_pin.take().unwrap();
    }

    fn get_spi_dc_pin(&mut self) -> Self::TSpiDC {
        return self.spi_dc_pin.take().unwrap();
    }

    fn get_display_rst_pin(&mut self) -> Self::TDisplayRst {
        return self.display_rst_pin.take().unwrap();
    }

    fn get_i2c_scl_pin(&mut self) -> Self::TI2cScl {
        return self.i2c_scl_pin.take().unwrap();
    }

    fn get_i2c_sda_pin(&mut self) -> Self::TI2cSda {
        return self.i2c_sda_pin.take().unwrap();
    }

    type TSpiSDO1 = Gpio0;

    type TSpiSDO2 = NotUsedPin;

    type TSpiSDO3 = NotUsedPin;

    type TDisplayEn = Gpio0;
}
