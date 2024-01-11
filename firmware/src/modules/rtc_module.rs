use crate::peripherals::i2c_proxy_async::I2cProxyAsync;
use esp_idf_hal::i2c::I2cDriver;
use log::info;
use time::{PrimitiveDateTime, UtcOffset};
use tokio::sync::broadcast::Sender;

use crate::peripherals::rtc::Rtc;
use log::debug;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;

pub struct RtcModule {}

#[link_section = ".rtc.data"]
static mut UTC_OFFSET: Option<UtcOffset> = None;

impl RtcModule {
    pub async fn start<'a>(
        proxy: I2cProxyAsync<I2cDriver<'a>>,
        commands: Sender<Commands>,
        events: Sender<Events>,
    ) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let mut rtc = Rtc::create(proxy);

        let mut timezone: Option<UtcOffset> = None;

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    debug!("{:?}", command);
                    match command {
                        Commands::GetTimeNow => {
                            unsafe {
                                    if timezone.is_none() && UTC_OFFSET.is_none() {
                                    continue;
                                }
                            }

                            let utc_offset = if timezone.is_none() {
                                unsafe {
                                    UTC_OFFSET.unwrap()
                                }
                            } else {
                                timezone.unwrap()
                            };

                            let datetime = rtc.get_now_utc()
                                .assume_offset(utc_offset);
                            events.send(Events::TimeNow(datetime)).unwrap();
                        }
                        Commands::SetTime(time) => {
                            let offset_utc = time.offset();

                            unsafe {
                                UTC_OFFSET = Some(time.offset());
                            }

                            timezone = Some(offset_utc);

                            let now = PrimitiveDateTime::new(time.date(), time.time());
                            rtc.set_now_utc(now).unwrap()
                        }
                        Commands::StartDeepSleep => {
                            break;
                        }
                        _ => {}
                    }
                }
                Ok(event) = recv_event.recv() => {
                    debug!("{:?}", event);

                    match event {
                        Events::Timezone(tz) => {
                            timezone = Some(UtcOffset::from_whole_seconds(tz).unwrap());

                            unsafe {
                                UTC_OFFSET = timezone;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        info!("done");
    }
}
