use std::cell::{Ref, RefCell, RefMut};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use embedded_hal::digital::OutputPin;
use embedded_hal_bus::i2c::RefCellDevice;
use esp_idf_hal::gpio::{AnyIOPin, Gpio21, Gpio25, Gpio26, IOPin, Output, OutputMode, Pin, PinDriver};
use esp_idf_hal::i2c::{I2c, I2C0, I2cConfig, I2cDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::SPI2;
use esp_idf_hal::units::FromValueType;

use crate::peripherals::accelerometer::Accelerometer;
use crate::peripherals::backlight::Backlight;
use crate::peripherals::bluetooth::{Bluetooth, BluetoothConfig};
use crate::peripherals::display::ClockDisplay;
use crate::peripherals::i2c_management::I2cManagement;
use crate::peripherals::i2c_proxy::I2cProxy;
use crate::peripherals::rtc::Rtc;
use crate::peripherals::touchpad::{Touchpad, TouchpadConfig};

pub type ClockBacklight<'d> = Backlight<PinDriver<'d, AnyIOPin, Output>>;

pub struct HAL<'d> {
    backlight: Rc<RefCell<ClockBacklight<'d>>>,
    display: Rc<RefCell<ClockDisplay<'d>>>,
    i2c_manager: I2cManagement<'d>,
    pub config: PinConfig
}

pub struct Devices<'d> {
    accelerometer: Rc<RefCell<Accelerometer<'d>>>,
    touchpad: Rc<RefCell<Touchpad<'d>>>,
    rtc: Rc<RefCell<Rtc<'d>>>,
    bluetooth: Rc<RefCell<Bluetooth>>
}

pub struct PinConfig {
    pub backlight: i32,
    pub touch_interrupt_pin: i32,
    pub touch_reset_pin: i32,
    pub ble_config: BluetoothConfig
}

impl<'d> HAL<'d> {
    fn init_backlight(backlight_pin: AnyIOPin) -> Backlight<PinDriver<'d, AnyIOPin, Output>> {
        let pin_driver = PinDriver::output(backlight_pin).unwrap();
        Backlight::create(pin_driver)
    }

    fn init_display() -> ClockDisplay<'d> {
        ClockDisplay::create()
    }

    fn init_i2c(i2c: I2C0) -> I2cManagement<'d> {
        let scl = unsafe { Gpio25::new() };
        let sda = unsafe { Gpio26::new() };
        let config = I2cConfig::new().baudrate(100.kHz().into());

        I2cManagement::create(i2c, scl.downgrade(), sda.downgrade(), config)
    }

    pub fn new(config: PinConfig, peripherals: Peripherals) -> HAL<'d> {
        let backlightPin = unsafe { AnyIOPin::new(config.backlight) };
        let backlight = Self::init_backlight(backlightPin);
        let display = Self::init_display();

        Self {
            display: Rc::new(RefCell::new(display)),
            backlight: Rc::new(RefCell::new(backlight)),
            i2c_manager: Self::init_i2c(peripherals.i2c0),
            config
        }
    }

    pub fn display<'b>(&'b mut self) -> Rc<RefCell<ClockDisplay<'d>>> {
        return Rc::clone(&self.display);
    }

    pub fn backlight<'b>(&'b mut self) -> Rc<RefCell<ClockBacklight<'d>>> {
        return Rc::clone(&self.backlight);
    }

    pub fn get_i2c_proxy(&self) -> Rc<RefCell<I2cDriver<'d>>> {
        return self.i2c_manager.get_proxy_ref().clone()
    }
}

impl<'d> Devices<'d> {
    pub fn new<'a>(hal: &'a HAL<'d>) -> Devices<'d> {
        let accel = Accelerometer::create(
            I2cProxy::new(hal.get_i2c_proxy().clone()),
            I2cProxy::new(hal.get_i2c_proxy().clone()));

        let config = TouchpadConfig {
            interrupt_pin: hal.config.touch_interrupt_pin,
            reset_pin: hal.config.touch_reset_pin
        };

        let touch = Touchpad::create(I2cProxy::new(hal.get_i2c_proxy().clone()), config);

        let rtc = Rtc::create(I2cProxy::new(hal.get_i2c_proxy().clone()));

        let bluetooth = Bluetooth::create(hal.config.ble_config);

        Self {
            accelerometer: Rc::new(RefCell::new(accel)),
            touchpad: Rc::new(RefCell::new(touch)),
            rtc: Rc::new(RefCell::new(rtc)),
            bluetooth: Rc::new(RefCell::new(bluetooth))
        }
    }
}