use cst816s::{TouchEvent, CST816S};
use embedded_hal_compat::{Reverse, ReverseCompat};
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::{AnyIOPin, Input, Output, PinDriver};
use esp_idf_hal::i2c::I2cDriver;
use log::debug;
use peripherals::i2c_proxy_async::I2cProxyAsync;

pub type TouchpadDevice<'d> = CST816S<
    Reverse<I2cProxyAsync<I2cDriver<'d>>>,
    PinDriver<'d, AnyIOPin, Input>,
    PinDriver<'d, AnyIOPin, Output>,
>;

pub struct Touchpad<'d> {
    device: TouchpadDevice<'d>,
}

pub struct TouchpadConfig {
    pub reset_pin: i32,
    pub interrupt_pin: i32,
}

impl<'d> Touchpad<'d> {
    pub fn create(proxy: I2cProxyAsync<I2cDriver<'d>>, config: TouchpadConfig) -> Self {
        let reset_pin = unsafe { AnyIOPin::new(config.reset_pin) };
        let interrupt_pin = unsafe { AnyIOPin::new(config.interrupt_pin) };

        let rst = PinDriver::output(reset_pin).unwrap();
        let int = PinDriver::input(interrupt_pin).unwrap();

        let mut touchpad = CST816S::new(proxy.reverse(), int, rst);

        let mut delay = Ets;

        touchpad.setup(&mut delay).unwrap();

        Self { device: touchpad }
    }

    pub fn try_get_pos(&mut self) -> Option<(i32, i32)> {
        debug!("reading touch event...");
        let touch_event_opt = self.device.read_one_touch_event(true);
        debug!("touch event {:?}", touch_event_opt);

        match touch_event_opt {
            Some(touch_event) => {
                let TouchEvent { x, y, .. } = touch_event;
                Some((x, y))
            }
            None => None,
        }
    }
}
