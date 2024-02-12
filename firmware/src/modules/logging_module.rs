use blinky_shared::{commands::Commands, events::Events};
use log::{error, info};
use tokio::sync::broadcast::Sender;

pub struct LoggingModule();

impl LoggingModule {
    pub async fn start(commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        loop {
            tokio::select! {
                 Ok(command) = recv_cmd.recv() => {
                     info!("{:?}", command);
                     match command {
                        Commands::StartDeepSleep => {
                            break;
                        }
                        _ => {}
                    }
                 }
                 Ok(event) = recv_event.recv() => {

                     match event {
                        Events::Restored(unit) => {
                            match unit.data {
                                Ok(buf) => {
                                    info!("Restored {} of {} bytes", unit.kind.as_ref(), buf.len());
                                },
                                Err(err) => {
                                    error!("Failed to restore {} error: {}", unit.kind.as_ref(), err);
                                }
                            }
                        }
                         _ => {
                            info!("{:?}", event);
                        }
                     }
                 }
            }
        }
    }
}
