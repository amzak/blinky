use esp_idf_hal::adc::attenuation::adc_atten_t_ADC_ATTEN_DB_11;
use esp_idf_hal::adc::config::Config;
use esp_idf_hal::adc::config::Resolution::Resolution10Bit;
use esp_idf_hal::adc::{AdcChannelDriver, AdcDriver, ADC1};
use esp_idf_hal::gpio::Gpio36;

pub struct AdcDevice<'d> {
    adc: AdcDriver<'d, ADC1>,
    channel: AdcChannelDriver<'d, adc_atten_t_ADC_ATTEN_DB_11, Gpio36>,
}

impl AdcDevice<'_> {
    pub fn new(adc_hal: ADC1, gpio36: Gpio36) -> Self {
        let config = Config::new().resolution(Resolution10Bit).calibration(true);

        let adc = AdcDriver::new(adc_hal, &config).unwrap();
        let channel = AdcChannelDriver::new(gpio36).unwrap();

        Self { adc, channel }
    }

    pub fn read(&mut self) -> u16 {
        let adc_res = self.adc.read(&mut self.channel).unwrap();
        adc_res
    }
}
