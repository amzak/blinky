use crate::peripherals::i2c_proxy_async::I2cProxyAsync;
use crate::peripherals::touchpad::TouchpadConfig;
use esp_idf_hal::i2c::I2cDriver;
use log::info;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;
use blinky_shared::message_bus::{BusHandler, BusSender, ContextStub, MessageBus};

#[derive(Clone, Debug)]
pub struct TouchPosition {
    pub x: i32,
    pub y: i32,
}

pub struct TouchModule {}

impl BusHandler<ContextStub> for TouchModule {
    async fn event_handler(bus: &BusSender, context: &mut ContextStub, event: Events) {
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
            */
            _ => {}
        }
    }

    async fn command_handler(bus: &BusSender, context: &mut ContextStub, command: Commands) {}
}

impl TouchModule {
    pub async fn start<'a>(
        config: TouchpadConfig,
        proxy: I2cProxyAsync<I2cDriver<'a>>,
        bus: MessageBus,
    ) {
        info!("starting...");

        //let touchpad = Touchpad::create(proxy, config);

        MessageBus::handle::<ContextStub, Self>(bus, ContextStub {}).await;

        info!("done.")
    }
}
