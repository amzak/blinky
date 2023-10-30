use std::thread;
use esp_idf_hal::gpio::{AnyInputPin, AnyIOPin, Input, InterruptType, Output, PinDriver, Pull};
use log::info;
use tokio::sync::broadcast::{Sender, Receiver};
use crate::peripherals::hal::{Commands, Events};

use tokio::sync::{AcquireError, Semaphore, SemaphorePermit, TryAcquireError};
use crate::error::Error;

pub struct UserInput {

}

impl UserInput {

    pub async fn start(commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let mut pin_driver = Self::setup_irq_driver();

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    match command {
                        _ => {}
                    }
                }
                Ok(_) = pin_driver.wait_for_falling_edge() => {
                    events.send(Events::TouchOrMove).unwrap();
                }
            }
        }

        info!("UserInput done.");
    }

    fn setup_irq_driver() -> PinDriver<'static, AnyIOPin, Input> {
        let irq_pin = unsafe { AnyIOPin::new(32) };
        let pin_driver = PinDriver::input(irq_pin).unwrap();
        pin_driver
    }
}
