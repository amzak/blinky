use std::cell::{Ref, RefCell, RefMut};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use embedded_hal::digital::OutputPin;
use embedded_hal_bus::i2c::{CriticalSectionDevice, RefCellDevice};
use esp_idf_hal::gpio::{AnyIOPin, Gpio21, Gpio25, Gpio26, IOPin, Output, OutputMode, Pin, PinDriver};
use esp_idf_hal::i2c::{I2c, I2C0, I2cConfig, I2cDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::SPI2;
use esp_idf_hal::units::FromValueType;
use time::{OffsetDateTime, PrimitiveDateTime};
use crate::modules::reference_time::ReferenceData;

use crate::peripherals::accelerometer::Accelerometer;
use crate::peripherals::backlight::Backlight;
use crate::peripherals::bluetooth::{Bluetooth, BluetoothConfig};
use crate::peripherals::display::ClockDisplay;
use crate::peripherals::i2c_management::I2cManagement;
use crate::peripherals::i2c_proxy_async::I2cProxyAsync;
use crate::peripherals::rtc::Rtc;
use crate::peripherals::touchpad::{Touchpad, TouchpadConfig};
use crate::peripherals::wifi::{Wifi, WifiConfig};

pub struct HAL<'d> {
    i2c_manager: I2cManagement<'d>,
    wifi: Rc<RefCell<Wifi>>,

    pub config: HalConfig
}

pub struct HalConfig {
    pub backlight: i32,
    pub touch_interrupt_pin: i32,
    pub touch_reset_pin: i32,
    pub ble_config: BluetoothConfig,
    pub wifi_config: WifiConfig
}

pub struct PinConfig {
    pub backlight: i32
}

impl<'d> HAL<'d> {
    fn init_display() -> ClockDisplay<'d> {
        ClockDisplay::create()
    }

    fn init_i2c(i2c: I2C0) -> I2cManagement<'d> {
        let scl = unsafe { Gpio25::new() };
        let sda = unsafe { Gpio26::new() };
        let config = I2cConfig::new().baudrate(100.kHz().into());

        I2cManagement::create(i2c, scl.downgrade(), sda.downgrade(), config)
    }

    pub fn new(config: HalConfig, peripherals: Peripherals) -> HAL<'d> {
        let wifi = Wifi::create(config.wifi_config.clone(), peripherals.modem);

        Self {
            i2c_manager: Self::init_i2c(peripherals.i2c0),
            wifi: Rc::new(RefCell::new(wifi)),
            config
        }
    }

    pub fn get_i2c_proxy_async(&self) -> I2cProxyAsync<I2cDriver<'d>> {
        return self.i2c_manager.get_proxy_ref_async();
    }

    pub fn get_touch_config(&self) -> TouchpadConfig {
        TouchpadConfig {
            interrupt_pin: self.config.touch_interrupt_pin,
            reset_pin: self.config.touch_reset_pin
        }
    }
}

#[derive(Clone, Debug)]
pub enum WakeupCause {
    Ext0,
    Ext1,
    Undef,
    Timer,
    Ulp
}

#[derive(Clone, Debug)]
pub struct TouchPosition {
    pub x: i32,
    pub y: i32
}

#[derive(Clone, Debug)]
pub enum Commands {
    RequestReferenceData,
    RequestBluetoothConnection,
    SyncRtc,
    GetTimeNow,
    GetReferenceTime,
    SetTime(OffsetDateTime)
}

#[derive(Clone, Debug)]
pub enum Events {
    TimeNow(OffsetDateTime),
    BluetoothConnected,
    ReferenceData(ReferenceData),
    ReferenceTime(OffsetDateTime),
    WakeupCause(WakeupCause),
    TouchOrMove,
    PowerDownTimer,
    ScreenOffTimer,
    TouchPos(TouchPosition),
    IncomingData(Vec<u8>)
}