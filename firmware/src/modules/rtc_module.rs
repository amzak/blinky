use std::thread;

use crate::peripherals::i2c_proxy_async::I2cProxyAsync;
use esp_idf_hal::i2c::I2cDriver;
use log::info;
use time::{PrimitiveDateTime, UtcOffset};
use tokio::runtime::Handle;
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc::{channel, Receiver};

use crate::peripherals::rtc::Rtc;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;

pub struct RtcModule {}

#[link_section = ".rtc.data"]
static mut UTC_OFFSET: Option<UtcOffset> = None;

impl RtcModule {
    pub async fn start(
        proxy: I2cProxyAsync<I2cDriver<'static>>,
        commands: Sender<Commands>,
        events: Sender<Events>,
    ) {
        let mut recv_cmd = commands.subscribe();

        let (tx, rx) = channel::<Commands>(10);

        let rtc_task = tokio::task::Builder::new()
            .name("rtc loop")
            .spawn_blocking(move || {
                Self::rtc_loop(events, rx, proxy);
            })
            .unwrap();

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    match command {
                        Commands::StartDeepSleep => {
                            tx.send(command).await.unwrap();
                            break;
                        }
                        _ => {
                            tx.send(command).await.unwrap();
                        }
                    }
                }
            }
        }

        rtc_task.await.unwrap();

        info!("done");
    }

    fn rtc_loop(
        events: Sender<Events>,
        mut rx: Receiver<Commands>,
        proxy: I2cProxyAsync<I2cDriver<'static>>,
    ) {
        let mut timezone: Option<UtcOffset> = None;

        let mut rtc = Rtc::create(proxy);

        let handle = Handle::current();

        loop {
            let command_future = rx.recv();

            let command_opt = handle.block_on(command_future);

            match command_opt {
                Some(command) => match command {
                    Commands::GetTimeNow => {
                        unsafe {
                            if timezone.is_none() && UTC_OFFSET.is_none() {
                                continue;
                            }
                        }

                        let utc_offset = if timezone.is_none() {
                            unsafe { UTC_OFFSET.unwrap() }
                        } else {
                            timezone.unwrap()
                        };

                        let datetime = rtc.get_now_utc().assume_offset(utc_offset);

                        let core = esp_idf_hal::cpu::core();
                        info!(
                            "sending TimeNow in thread {:?} core {:?} queue {:?}",
                            thread::current().id(),
                            core,
                            events.len()
                        );

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
                    Commands::SetTimezone(tz) => {
                        timezone = Some(UtcOffset::from_whole_seconds(tz).unwrap());

                        unsafe {
                            UTC_OFFSET = timezone;
                        }
                    }
                    Commands::StartDeepSleep => {
                        break;
                    }
                    _ => {}
                },
                None => break,
            }
        }

        info!("rtc loop done.")
    }
}
