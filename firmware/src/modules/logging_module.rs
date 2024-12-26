use std::future::Future;

use blinky_shared::{commands::Commands, events::Events, message_bus::BusSender};
use log::{error, info};

use blinky_shared::message_bus::{BusHandler, ContextStub, MessageBus};

pub struct LoggingModule();

impl BusHandler<ContextStub> for LoggingModule {
    async fn event_handler(_bus: &BusSender, _context: &mut ContextStub, event: Events) {
        match event {
            Events::Restored(unit) => match unit.data {
                Ok(buf) => {
                    info!("Restored {} of {} bytes", unit.kind.as_ref(), buf.len());
                }
                Err(err) => {
                    error!("Failed to restore {} error: {}", unit.kind.as_ref(), err);
                }
            },
            Events::IncomingData(data) => {
                info!("IncomingData {:02X?}", &data);
            }
            _ => {
                info!("{:?}", event);
            }
        }
    }

    async fn command_handler(_bus: &BusSender, _context: &mut ContextStub, command: Commands) {
        info!("{:?}", command);
    }
}

impl LoggingModule {
    pub async fn start(bus: MessageBus) {
        info!("starting...");
        let context = ContextStub {};

        MessageBus::handle::<ContextStub, Self>(bus, context).await;
    }
}
