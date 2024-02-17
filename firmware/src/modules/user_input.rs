use std::time::Duration;

use esp_idf_hal::gpio::{AnyIOPin, Input, PinDriver};
use esp_idf_sys::EspError;
use log::info;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;

use blinky_shared::message_bus::{BusHandler, BusSender, ContextStub, MessageBus};
use tokio::time::sleep;

pub struct UserInput {}

impl BusHandler<ContextStub> for UserInput {
    async fn event_handler(bus: &BusSender, context: &mut ContextStub, event: Events) {}

    async fn command_handler(bus: &BusSender, context: &mut ContextStub, command: Commands) {}
}

impl UserInput {
    pub async fn start(mut bus: MessageBus) {
        info!("starting...");

        let pin_driver = Self::setup_irq_driver();

        let bus_clone = bus.clone();
        let task = tokio::spawn(Self::wait_for_touch(pin_driver, bus_clone));

        MessageBus::handle::<ContextStub, Self>(bus, ContextStub {}).await;

        task.abort();

        info!("done.");
    }

    async fn wait_for_touch(
        mut pin: PinDriver<'static, AnyIOPin, Input>,
        bus: MessageBus,
    ) -> Result<(), EspError> {
        loop {
            pin.wait_for_falling_edge().await.unwrap();
            bus.send_event(Events::TouchOrMove);
            sleep(Duration::from_millis(50)).await;
        }
    }

    fn setup_irq_driver() -> PinDriver<'static, AnyIOPin, Input> {
        let irq_pin = unsafe { AnyIOPin::new(32) };
        let pin_driver = PinDriver::input(irq_pin).unwrap();
        pin_driver
    }
}
