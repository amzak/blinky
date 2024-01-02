use tokio::sync::broadcast::Sender;

use crate::peripherals::hal::{Commands, Events};

pub trait Module {
    fn new() -> Self;
    async fn start(&self, commands: Sender<Commands>, events: Sender<Events>);
}
