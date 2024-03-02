use crate::peripherals::i2c_proxy_async::I2cProxyAsync;
use esp_idf_hal::i2c::I2cDriver;
use log::info;
use time::{PrimitiveDateTime, UtcOffset};
use tokio::runtime::Handle;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::peripherals::rtc::Rtc;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;
use blinky_shared::message_bus::{BusHandler, BusSender, MessageBus};

pub struct RtcModule {}

struct Context {
    tx: Sender<Commands>,
}

#[link_section = ".rtc.data"]
static mut UTC_OFFSET: Option<UtcOffset> = None;

impl BusHandler<Context> for RtcModule {
    async fn event_handler(_bus: &BusSender, _context: &mut Context, _event: Events) {}

    async fn command_handler(_bus: &BusSender, context: &mut Context, command: Commands) {
        match command {
            _ => {
                context.tx.send(command).await.unwrap();
            }
        }
    }
}

impl RtcModule {
    pub async fn start(proxy: I2cProxyAsync<I2cDriver<'static>>, mut bus: MessageBus) {
        info!("starting...");
        let (tx, rx) = channel::<Commands>(10);

        let bus_clone = bus.clone();
        let rtc_task = tokio::task::spawn_blocking(move || {
            Self::rtc_loop(bus_clone, rx, proxy);
        });

        let context = Context { tx };

        MessageBus::handle::<Context, Self>(bus, context).await;

        rtc_task.await.unwrap();

        info!("done.");
    }

    fn rtc_loop(
        bus: MessageBus,
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

                        bus.send_event(Events::TimeNow(datetime));
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
