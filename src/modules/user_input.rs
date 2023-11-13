use esp_idf_hal::gpio::{AnyIOPin, Input, PinDriver};
use esp_idf_sys::EspError;
use log::info;
use tokio::sync::broadcast::Sender;
use crate::peripherals::hal::{Commands, Events};
use tokio::time::{sleep, Duration};

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
                Ok(_) = Self::wait_for_touch(&mut pin_driver) => {
                    let level = pin_driver.get_level();
                    info!("pin irq level {:?}", level);
                    events.send(Events::TouchOrMove).unwrap();
                }
            }
        }

        info!("done.");
    }

    async fn wait_for_touch(pin: &mut PinDriver<'static, AnyIOPin, Input>) -> Result<(), EspError> {
        sleep(Duration::from_millis(5)).await;
        let level = pin.get_level();
        info!("waiting for input... {:?}", level);
        return pin.wait_for_falling_edge().await;
    }

    fn setup_irq_driver() -> PinDriver<'static, AnyIOPin, Input> {
        let irq_pin = unsafe { AnyIOPin::new(32) };
        let pin_driver = PinDriver::input(irq_pin).unwrap();
        pin_driver
    }
}
