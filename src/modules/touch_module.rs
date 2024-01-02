use crate::peripherals::hal::{Commands, Events};
use crate::peripherals::i2c_proxy_async::I2cProxyAsync;
use crate::peripherals::touchpad::{Touchpad, TouchpadConfig};
use esp_idf_hal::i2c::I2cDriver;
use log::info;
use tokio::sync::broadcast::Sender;

#[derive(Clone, Debug)]
pub struct TouchPosition {
    pub x: i32,
    pub y: i32,
}

pub struct TouchModule {}

impl TouchModule {
    pub async fn start<'a>(
        config: TouchpadConfig,
        proxy: I2cProxyAsync<I2cDriver<'a>>,
        commands: Sender<Commands>,
        events: Sender<Events>,
    ) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();
        let mut touchpad = Touchpad::create(proxy, config);

        loop {
            tokio::select! {
                            Ok(command) = recv_cmd.recv() => {
                                match command {
                                    Commands::StartDeepSleep => {
                                        break;
                                    }
                                    _ => {}
                                }
                            },
                            Ok(event) = recv_event.recv() => {
                                match event {
            /*                        Events::TouchOrMove => {
                                        if let Some((x,y)) = touchpad.try_get_pos() {
                                            let pos = TouchPosition {
                                                x,
                                                y
                                            };
                                            events.send(Events::TouchPos(pos)).unwrap();
                                        }
                                    }
            */                        _ => {}
                                }
                            },
                        }
        }

        info!("done.")
    }
}
