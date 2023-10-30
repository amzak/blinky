use embedded_hal_bus::i2c::CriticalSectionDevice;
use esp_idf_hal::i2c::{I2cDriver, I2cError};
use embedded_hal_compat::{Reverse, ReverseCompat};
use embedded_svc::event_bus::EventBus;
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, UtcOffset};
use tokio::sync::broadcast::Sender;
use crate::peripherals::hal::{Commands, Events};

use crate::peripherals::i2c_proxy_async::I2cProxyAsync;
use crate::peripherals::rtc::Rtc;

pub struct RtcModule {
}

type Error<'a> = &'a str;

impl RtcModule {

    pub async fn start<'a>(proxy: I2cProxyAsync<I2cDriver<'a>>, commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let mut rtc = Rtc::create(proxy);

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    match command {
                        Commands::GetTimeNow => {
                            let datetime = rtc.get_now();
                            events.send(Events::TimeNow(datetime)).unwrap();
                        }
                        Commands::SetTime(time) => {
                            let now = PrimitiveDateTime::new(time.date(), time.time());
                            rtc.set_now(now).unwrap()
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
