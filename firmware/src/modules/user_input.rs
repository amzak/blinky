use std::time::Duration;

use esp_idf_hal::gpio::{AnyIOPin, Input, PinDriver};
use esp_idf_sys::EspError;
use log::info;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;

use blinky_shared::message_bus::{BusHandler, BusSender, ContextStub, MessageBus};
use tokio::select;
use tokio::time::sleep;

use crate::peripherals::output::PinOutput;

pub struct UserInput {}

impl BusHandler<ContextStub> for UserInput {
    async fn event_handler(_bus: &BusSender, _context: &mut ContextStub, _event: Events) {}

    async fn command_handler(_bus: &BusSender, _context: &mut ContextStub, _command: Commands) {}
}

impl UserInput {
    pub async fn start(bus: MessageBus) {
        info!("starting...");

        let bus_clone = bus.clone();
        let task = tokio::spawn(Self::wait_for_touch(bus_clone));

        MessageBus::handle::<ContextStub, Self>(bus, ContextStub {}).await;

        task.abort();

        info!("done.");
    }

    async fn wait_for_touch(bus: MessageBus) -> Result<(), EspError> {
        let mut pin_touch = Self::setup_irq_driver(32);
        let mut pin_key1 = Self::setup_irq_driver(34);
        let mut pin_key2 = Self::setup_irq_driver(35);

        let mut vibro = PinOutput::create(4, false);

        loop {
            select! {
                Ok(_) = pin_touch.wait_for_low() => {
                    bus.send_event(Events::SharedInterrupt);
                }
                Ok(_) = pin_key1.wait_for_low() => {
                    bus.send_event(Events::Key1Press);
                }
                Ok(_) = pin_key2.wait_for_low() => {
                    bus.send_event(Events::Key2Press);
                    vibro.on();
                    sleep(Duration::from_millis(200)).await;
                    vibro.off();
                }
            }
            sleep(Duration::from_millis(300)).await;
        }
    }

    fn setup_irq_driver(pin_id: i32) -> PinDriver<'static, AnyIOPin, Input> {
        let irq_pin = unsafe { AnyIOPin::new(pin_id) };
        let pin_driver = PinDriver::input(irq_pin).unwrap();
        pin_driver
    }
}
