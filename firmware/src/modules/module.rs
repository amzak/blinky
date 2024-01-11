use blinky_shared::{commands::Commands, events::Events};
use tokio::sync::broadcast::Sender;

pub trait Module {
    fn new() -> Self;
    async fn start(&self, commands: Sender<Commands>, events: Sender<Events>);
}
