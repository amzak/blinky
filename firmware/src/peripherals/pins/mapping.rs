use esp_idf_hal::gpio::{ADCPin, IOPin, OutputPin, Pins};

pub trait PinsMapping {
    type TAdcPin: ADCPin;
    type TBacklightPin: OutputPin;

    type TSpiCSPin: OutputPin;
    type TSpiSCLK: OutputPin;
    type TSpiSDO: OutputPin;
    type TSpiDC: embedded_hal::digital::OutputPin;

    type TDisplayRst: embedded_hal::digital::OutputPin;

    type TI2cScl: IOPin;
    type TI2cSda: IOPin;

    fn new(peripherals: Pins) -> Self;

    fn get_adc_pin(&mut self) -> Self::TAdcPin;

    fn get_backlight_pin(&mut self) -> &Self::TBacklightPin;

    fn get_spi_cs_pin(&mut self) -> Self::TSpiCSPin;

    fn get_spi_sclk_pin(&mut self) -> Self::TSpiSCLK;

    fn get_spi_sdo_pin(&mut self) -> Self::TSpiSDO;

    fn get_spi_dc_pin(&mut self) -> Self::TSpiDC;

    fn get_display_rst_pin(&mut self) -> Self::TDisplayRst;

    fn get_i2c_scl_pin(&mut self) -> Self::TI2cScl;

    fn get_i2c_sda_pin(&mut self) -> Self::TI2cSda;
}
