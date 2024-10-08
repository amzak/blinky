use crate::peripherals::i2c_management::I2cManagement;
use crate::peripherals::i2c_proxy_async::I2cProxyAsync;
use crate::peripherals::touchpad::TouchpadConfig;
use esp_idf_hal::gpio::{Gpio25, Gpio26, IOPin};
use esp_idf_hal::i2c::{I2cConfig, I2cDriver, I2C0};
use esp_idf_hal::units::FromValueType;

pub struct HAL<'d> {
    i2c_manager: I2cManagement<'d>,
    pub config: HalConfig,
}

#[derive(Clone, Copy)]
pub struct HalConfig {
    pub backlight: i32,
    pub touch_interrupt_pin: i32,
    pub touch_reset_pin: i32,
}

pub struct PinConfig {
    pub backlight: i32,
}

impl<'d> HAL<'d> {
    fn init_i2c(i2c: I2C0) -> I2cManagement<'d> {
        let scl = unsafe { Gpio25::new() };
        let sda = unsafe { Gpio26::new() };
        let config = I2cConfig::new().baudrate(100.kHz().into());

        I2cManagement::create(i2c, scl.downgrade(), sda.downgrade(), config)
    }

    pub fn new(config: HalConfig, peripherals: I2C0) -> HAL<'d> {
        Self {
            i2c_manager: Self::init_i2c(peripherals),
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
