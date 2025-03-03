use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_hal::adc::Adc;
use esp_idf_hal::gpio::ADCPin;
use esp_idf_hal::peripheral::Peripheral;

pub struct AdcDevice<'d, TAdcPin>
where
    TAdcPin: ADCPin,
    TAdcPin::Adc: Adc,
{
    channel: AdcChannelDriver<'d, TAdcPin, AdcDriver<'d, TAdcPin::Adc>>,
}

impl<'d, TAdcPin> AdcDevice<'d, TAdcPin>
where
    TAdcPin: ADCPin,
    TAdcPin::Adc: Adc,
{
    pub fn new(
        adc_hal: impl Peripheral<P = TAdcPin::Adc> + 'd,
        pin: impl Peripheral<P = TAdcPin> + 'd,
    ) -> Self {
        #[cfg(feature = "twatch_2021")]
        let config = AdcChannelConfig {
            resolution: esp_idf_hal::adc::Resolution::Resolution10Bit,
            attenuation: DB_11,
            ..Default::default()
        };

        #[cfg(feature = "tdisplay143")]
        let config = AdcChannelConfig {
            resolution: esp_idf_hal::adc::Resolution::Resolution12Bit,
            attenuation: DB_11,
            ..Default::default()
        };

        let adc = AdcDriver::new(adc_hal).unwrap();
        let channel = AdcChannelDriver::new(adc, pin, &config).unwrap();

        Self { channel }
    }

    pub fn read(&mut self) -> u16 {
        let adc_res = self.channel.read().unwrap();
        adc_res
    }
}
