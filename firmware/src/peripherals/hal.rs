use crate::peripherals::touchpad::TouchpadConfig;
use esp_idf_hal::gpio::IOPin;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver, I2C0};
use esp_idf_hal::units::FromValueType;
use peripherals::i2c_management::I2cManagement;
use peripherals::i2c_proxy_async::I2cProxyAsync;

use super::pins::mapping::PinsMapping;

pub struct HAL<'d> {
    i2c_manager: I2cManagement<'d>,
    pub config: HalConfig,
}

#[derive(Clone, Copy)]
pub struct HalConfig {
    pub touch_interrupt_pin: i32,
    pub touch_reset_pin: i32,
}

pub struct PinConfig {
    pub vibro: i32,
}

impl<'d> HAL<'d> {
    pub fn new<TI2cScl, TI2cSda, PM>(config: HalConfig, i2c: I2C0, pins_mapping: &mut PM) -> HAL<'d>
    where
        TI2cScl: IOPin,
        TI2cSda: IOPin,
        PM: PinsMapping<TI2cScl = TI2cScl, TI2cSda = TI2cSda>,
    {
        let scl = pins_mapping.get_i2c_scl_pin();
        let sda = pins_mapping.get_i2c_sda_pin();
        let i2c_config = I2cConfig::new().baudrate(100.kHz().into());

        let i2c_management =
            I2cManagement::create(i2c, scl.downgrade(), sda.downgrade(), i2c_config);

        Self {
            i2c_manager: i2c_management,
            config,
        }
    }

    pub fn get_i2c_proxy_async(&self) -> I2cProxyAsync<I2cDriver<'d>> {
        return self.i2c_manager.get_proxy_ref_async();
    }

    pub fn get_touch_config(&self) -> TouchpadConfig {
        TouchpadConfig {
            interrupt_pin: self.config.touch_interrupt_pin,
            reset_pin: self.config.touch_reset_pin,
        }
    }
}
