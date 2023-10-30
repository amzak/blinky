use esp_idf_hal::i2c::I2cDriver;
use tokio::sync::broadcast::{Sender, Receiver};
use crate::peripherals::accelerometer::Accelerometer;
use crate::peripherals::hal::{Commands, Events};
use crate::peripherals::i2c_proxy_async::I2cProxyAsync;

pub struct AccelerometerModule {

}

impl AccelerometerModule {
    pub async fn start<'a>(proxy: I2cProxyAsync<I2cDriver<'a>>, proxy_ex: I2cProxyAsync<I2cDriver<'a>>, commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let accel = Accelerometer::create(proxy, proxy_ex);

        loop {
            tokio::select! {
                Ok(event) = recv_event.recv() => {
                    match event {
                        Events::TouchOrMove => {

                        }
                        _ => {}
                    }
                },
            }
        }
    }
}