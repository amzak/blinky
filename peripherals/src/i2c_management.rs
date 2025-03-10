use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::i2c::config::Config;
use esp_idf_hal::i2c::{I2c, I2cDriver};
use esp_idf_hal::peripheral::Peripheral;

use crate::i2c_proxy_async::I2cProxyAsync;

pub struct I2cManagement<'a> {
    i2c: I2cProxyAsync<I2cDriver<'a>>,
}

impl<'a> I2cManagement<'a> {
    pub fn create<I2C: I2c>(
        i2c: impl Peripheral<P = I2C> + 'a,
        scl: AnyIOPin,
        sda: AnyIOPin,
        config: Config,
    ) -> Self {
        let i2c_driver = I2cProxyAsync::new(I2cDriver::new(i2c, sda, scl, &config).unwrap());
        Self { i2c: i2c_driver }
    }

    pub fn get_proxy_ref_async(&self) -> I2cProxyAsync<I2cDriver<'a>> {
        self.i2c.clone()
    }
}
