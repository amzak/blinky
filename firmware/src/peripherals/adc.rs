use esp_idf_hal::adc::attenuation::adc_atten_t_ADC_ATTEN_DB_11;
use esp_idf_hal::adc::config::Config;
use esp_idf_hal::adc::config::Resolution::Resolution10Bit;
use esp_idf_hal::adc::Adc;
use esp_idf_hal::adc::{AdcChannelDriver, AdcDriver};
use esp_idf_hal::gpio::ADCPin;
use esp_idf_hal::peripheral::Peripheral;

pub struct AdcDevice<'d, TAdcPin>
where
    TAdcPin: ADCPin,
    TAdcPin::Adc: Adc,
{
    adc: AdcDriver<'d, TAdcPin::Adc>,
    channel: AdcChannelDriver<'d, adc_atten_t_ADC_ATTEN_DB_11, TAdcPin>,
}

impl<'d, TAdcPin> AdcDevice<'d, TAdcPin>
where
    TAdcPin: ADCPin<Adc: Adc>,
    TAdcPin::Adc: Adc,
{
    pub fn new(
        adc_hal: impl Peripheral<P = TAdcPin::Adc> + 'd,
        pin: impl Peripheral<P = TAdcPin> + 'd,
    ) -> Self {
        let config = Config::new().resolution(Resolution10Bit).calibration(true);

        let adc = AdcDriver::new(adc_hal, &config).unwrap();
        let channel: AdcChannelDriver<'_, adc_atten_t_ADC_ATTEN_DB_11, TAdcPin> =
            AdcChannelDriver::new(pin).unwrap();

        Self { adc, channel }
    }

    pub fn read(&mut self) -> u16 {
        let adc_res = self.adc.read(&mut self.channel).unwrap();
        adc_res
    }
}
