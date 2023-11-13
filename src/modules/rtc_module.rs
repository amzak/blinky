use esp_idf_hal::i2c::I2cDriver;
use log::info;
use time::PrimitiveDateTime;
use tokio::sync::broadcast::Sender;
use crate::peripherals::hal::{Commands, Events};
use crate::peripherals::i2c_proxy_async::I2cProxyAsync;

use crate::peripherals::rtc::Rtc;
use log::debug;

pub struct RtcModule {
}

impl RtcModule {

    pub async fn start<'a>(proxy: I2cProxyAsync<I2cDriver<'a>>, commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let mut rtc = Rtc::create(proxy);

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    debug!("{:?}", command);
                    match command {
                        Commands::GetTimeNow => {
                            let datetime = rtc.get_now();
                            events.send(Events::TimeNow(datetime)).unwrap();
                        }
                        Commands::SetTime(time) => {
                            let now = PrimitiveDateTime::new(time.date(), time.time());
                            rtc.set_now(now).unwrap()
                        }
                        Commands::StartDeepSleep => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        info!("done");
    }
}
