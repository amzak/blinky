use esp_idf_hal::i2c::I2cDriver;
use tokio::sync::broadcast::{Sender, Receiver};
use crate::peripherals::accelerometer::Accelerometer;
use crate::peripherals::hal::{Commands, Events, TouchPosition};
use crate::peripherals::i2c_proxy_async::I2cProxyAsync;
use crate::peripherals::touchpad::{Touchpad, TouchpadConfig};

pub struct TouchModule {

}

impl TouchModule {
    pub async fn start<'a>(config: TouchpadConfig, proxy: I2cProxyAsync<I2cDriver<'a>>, commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let mut touchpad = Touchpad::create(proxy, config);

        loop {
            tokio::select! {
                Ok(event) = recv_event.recv() => {
                    match event {
                        Events::TouchOrMove => {
                            if let Some((x,y)) = touchpad.try_get_pos() {
                                let pos = TouchPosition {
                                    x,
                                    y
                                };
                                events.send(Events::TouchPos(pos)).unwrap();
                            }
                        }
                        _ => {}
                    }
                },
            }
        }
    }
}