    use std::borrow::Borrow;
use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use embedded_hal_bus::i2c::{CriticalSectionDevice, RefCellDevice};
    use esp_idf_hal::gpio::{AnyInputPin, AnyIOPin, Gpio25, Gpio26, InputOutput, Pin};
use esp_idf_hal::i2c::{I2C0, I2cConfig, I2cDriver};
use esp_idf_hal::i2c::config::Config;
    use crate::peripherals::i2c_proxy_async::I2cProxyAsync;

    pub struct I2cManagement<'a> {
    i2c: I2cProxyAsync<I2cDriver<'a>>
}

impl<'a> I2cManagement<'a> {
    pub fn create(i2c: I2C0, scl: AnyIOPin, sda: AnyIOPin, config: Config) -> Self {
        let i2c_driver = I2cProxyAsync::new(I2cDriver::new(i2c, sda, scl, &config).unwrap());
        Self {
            i2c: i2c_driver,
        }
    }

    pub fn get_proxy_ref_async(&self) -> I2cProxyAsync<I2cDriver<'a>> {
        self.i2c.clone()
    }
}