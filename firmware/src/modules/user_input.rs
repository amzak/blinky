use esp_idf_hal::gpio::{AnyIOPin, Input, PinDriver};
use esp_idf_sys::EspError;
use log::info;
use tokio::sync::broadcast::Sender;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;

pub struct UserInput {}

impl UserInput {
    pub async fn start(commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let pin_driver = Self::setup_irq_driver();

        let task = tokio::spawn(Self::wait_for_touch(pin_driver, events));

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    match command {
                        Commands::StartDeepSleep => {
                            break;
                        }
                        _ => {}
                    }
                }
                Ok(event) = recv_event.recv() => {
                    match event {
                        _ => {}
                    }
                }
            }
        }

        task.abort();

        info!("done.");
    }

    async fn wait_for_touch(
        mut pin: PinDriver<'static, AnyIOPin, Input>,
        events: Sender<Events>,
    ) -> Result<(), EspError> {
        loop {
            pin.wait_for_falling_edge().await.unwrap();
            events.send(Events::TouchOrMove).unwrap();
        }
    }

    fn setup_irq_driver() -> PinDriver<'static, AnyIOPin, Input> {
        let irq_pin = unsafe { AnyIOPin::new(32) };
        let pin_driver = PinDriver::input(irq_pin).unwrap();
        pin_driver
    }
}
