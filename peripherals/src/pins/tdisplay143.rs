use esp_idf_hal::gpio::{
    Gpio0, Gpio10, Gpio11, Gpio12, Gpio13, Gpio14, Gpio15, Gpio16, Gpio17, Gpio4, Gpio6, Gpio7,
    Gpio9, Output, Pin, PinDriver,
};

use super::mapping::{NotUsedPin, PinsMapping};

pub struct TDisplay143 {
    adc_pin: Option<Gpio4>,
    backlight_pin: Option<Gpio16>,
    spi_cs_pin: Option<Gpio10>,
    spi_sclk_pin: Option<Gpio12>,
    spi_sdo_pin: Option<Gpio11>,
    spi_dc_pin: Option<NotUsedPin>,
    display_rst_pin: Option<PinDriver<'static, Gpio17, Output>>,
    i2c_scl_pin: Option<Gpio6>,
    i2c_sda_pin: Option<Gpio7>,

    spi_sdo1_pin: Option<Gpio13>,
    spi_sdo2_pin: Option<Gpio14>,
    spi_sdo3_pin: Option<Gpio15>,
    display_en: Option<Gpio16>,

    touch_int: Option<Gpio9>,
    button1: Option<Gpio0>,
}

impl TDisplay143 {
    pub fn new(peripherals: esp_idf_hal::gpio::Pins) -> Self {
        TDisplay143 {
            adc_pin: Some(peripherals.gpio4),
            backlight_pin: None,
            spi_cs_pin: Some(peripherals.gpio10),
            spi_sclk_pin: Some(peripherals.gpio12),
            spi_sdo_pin: Some(peripherals.gpio11),
            spi_dc_pin: None,
            display_rst_pin: Some(PinDriver::output(peripherals.gpio17).unwrap()),
            i2c_scl_pin: Some(peripherals.gpio6),
            i2c_sda_pin: Some(peripherals.gpio7),
            spi_sdo1_pin: Some(peripherals.gpio13),
            spi_sdo2_pin: Some(peripherals.gpio14),
            spi_sdo3_pin: Some(peripherals.gpio15),
            display_en: Some(peripherals.gpio16),
            touch_int: Some(peripherals.gpio9),
            button1: Some(peripherals.gpio0),
        }
    }
}

impl PinsMapping for TDisplay143 {
    type TAdcPin = Gpio4;

    type TBacklightPin = Gpio16;

    type TSpiCSPin = Gpio10;
    type TSpiSCLK = Gpio12;
    type TSpiSDO = Gpio11;
    type TSpiSDO1 = Gpio13;
    type TSpiSDO2 = Gpio14;
    type TSpiSDO3 = Gpio15;

    type TSpiDC = NotUsedPin; // NOT USED

    type TDisplayRst = PinDriver<'static, Gpio17, Output>;
    type TDisplayEn = Gpio16;

    type TI2cScl = Gpio6;
    type TI2cSda = Gpio7;

    type TTouchInterrupt = Gpio9;

    type TButton1 = Gpio0;

    fn get_adc_pin(&mut self) -> Self::TAdcPin {
        self.adc_pin.take().unwrap()
    }

    fn get_backlight_pin(&mut self) -> Option<Self::TBacklightPin> {
        self.backlight_pin.take()
    }

    fn get_spi_cs_pin(&mut self) -> Self::TSpiCSPin {
        self.spi_cs_pin.take().unwrap()
    }

    fn get_spi_sclk_pin(&mut self) -> Self::TSpiSCLK {
        self.spi_sclk_pin.take().unwrap()
    }

    fn get_spi_sdo_pin(&mut self) -> Self::TSpiSDO {
        self.spi_sdo_pin.take().unwrap()
    }

    fn get_spi_dc_pin(&mut self) -> Self::TSpiDC {
        self.spi_dc_pin.take().unwrap()
    }

    fn get_display_rst_pin(&mut self) -> Self::TDisplayRst {
        self.display_rst_pin.take().unwrap()
    }

    fn get_i2c_scl_pin(&mut self) -> Self::TI2cScl {
        self.i2c_scl_pin.take().unwrap()
    }

    fn get_i2c_sda_pin(&mut self) -> Self::TI2cSda {
        self.i2c_sda_pin.take().unwrap()
    }

    fn get_spi_sdo1_pin(&mut self) -> Self::TSpiSDO1 {
        self.spi_sdo1_pin.take().unwrap()
    }

    fn get_spi_sdo2_pin(&mut self) -> Self::TSpiSDO2 {
        self.spi_sdo2_pin.take().unwrap()
    }

    fn get_spi_sdo3_pin(&mut self) -> Self::TSpiSDO3 {
        self.spi_sdo3_pin.take().unwrap()
    }

    fn get_display_en_pin(&mut self) -> Self::TDisplayEn {
        self.display_en.take().unwrap()
    }

    fn get_touch_int_pin(&mut self) -> Self::TTouchInterrupt {
        self.touch_int.take().unwrap()
    }

    fn get_button1_pin(&mut self) -> Self::TButton1 {
        self.button1.take().unwrap()
    }

    fn get_touch_int_pin_index(&self) -> i32 {
        self.touch_int.as_ref().expect("no pin").pin()
    }

    fn get_button1_pin_index(&self) -> i32 {
        self.button1.as_ref().expect("no pin").pin()
    }
}
