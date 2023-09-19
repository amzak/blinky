use std::thread;
use std::time::Duration;
use cst816s::{CST816S, TouchEvent};
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::{AnyIOPin, Input, Output, PinDriver};
use esp_idf_hal::i2c::{I2c, I2cDriver};
use embedded_hal_compat::{Reverse, ReverseCompat};

use crate::peripherals::i2c_proxy::I2cProxy;

pub type TouchpadDevice<'d> = CST816S<Reverse<I2cProxy<I2cDriver<'d>>>, PinDriver<'d, AnyIOPin, Input>, PinDriver<'d, AnyIOPin, Output>>;

pub struct Touchpad<'d> {
    device: TouchpadDevice<'d>
}

pub struct TouchpadConfig {
    pub reset_pin: i32,
    pub interrupt_pin: i32
}

impl<'d> Touchpad<'d> {
    pub fn create(proxy: I2cProxy<I2cDriver<'d>>, config: TouchpadConfig) -> Self {
        let reset_pin = unsafe { AnyIOPin::new(config.reset_pin) };
        let interrupt_pin = unsafe { AnyIOPin::new(config.interrupt_pin) };

        let rst = PinDriver::output(reset_pin).unwrap();
        let int = PinDriver::input(interrupt_pin).unwrap();

        let mut touchpad = CST816S::new(proxy.reverse(), int, rst);

        let mut delay = Ets;

        touchpad.setup(&mut delay).unwrap();

        //let info = touchpad.get_device_info().unwrap();
        //println!("touch device version = {} info = {:02X?}", info.Version, info.VersionInfo);

        Self {
            device: touchpad
        }
    }

    pub fn test(&mut self) {
        let mut data: [u8; 10] = [0; 10];

        for i in 0..100 {
            //self.device.get_data_raw(&mut data).unwrap();
            //println!("touch raw data = {:02X?}", data);

            let mut device = &mut self.device;
            let touch_event = device.read_one_touch_event(false).unwrap();
            let TouchEvent {x,y,..} = touch_event;

            println!("touch gesture = {:?} x = {} y = {}", touch_event.gesture, x, y);

            thread::sleep(Duration::from_millis(20));
        }
    }
}