use log::info;
use time::{PrimitiveDateTime, UtcOffset};
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::watch;
use tokio::time::MissedTickBehavior;

use crate::peripherals::rtc::Rtc;
use crate::peripherals::rtc_memory::UTC_OFFSET;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;
use blinky_shared::message_bus::{BusHandler, BusSender, MessageBus};

pub struct RtcModule {}

struct Context {
    tx_rtc: Sender<Commands>,
}

impl BusHandler<Context> for RtcModule {
    async fn event_handler(_bus: &BusSender, _context: &mut Context, _event: Events) {}

    async fn command_handler(_bus: &BusSender, context: &mut Context, command: Commands) {
        match command {
            _ => {
                context.tx_rtc.send(command).await.unwrap();
            }
        }
    }
}

impl RtcModule {
    pub async fn start(rtc: Rtc<'static>, bus: MessageBus) {
        info!("starting...");
        let (tx_rtc, rx_rtc) = channel::<Commands>(10);

        let (tx_timer, rx_timer) = watch::channel(true);

        let bus_clone = bus.clone();

        let rtc_task = tokio::task::spawn_blocking(move || {
            Self::rtc_loop(bus_clone, rx_rtc, tx_timer, rtc);
        });

        let timer = tokio::spawn(Self::run_timer(rx_timer, tx_rtc.clone()));

        let context = Context { tx_rtc };

        MessageBus::handle::<Context, Self>(bus, context).await;

        rtc_task.await.unwrap();

        timer.abort();

        info!("done.");
    }

    fn rtc_loop(
        bus: MessageBus,
        mut rx: Receiver<Commands>,
        tx_timer: tokio::sync::watch::Sender<bool>,
        rtc_param: Rtc,
    ) {
        let mut timezone: UtcOffset = UtcOffset::from_whole_seconds(0).unwrap();

        let mut rtc = rtc_param;

        let utc_offset = unsafe {
            if UTC_OFFSET.is_none() {
                timezone
            } else {
                UTC_OFFSET.unwrap()
            }
        };

        let datetime = rtc.get_now_utc().assume_offset(utc_offset);
        bus.send_event(Events::TimeNow(datetime));

        loop {
            let command_opt = rx.blocking_recv();

            match command_opt {
                Some(command) => match command {
                    Commands::GetTimeNow => {
                        let datetime = rtc.get_now_utc().assume_offset(utc_offset);
                        bus.send_event(Events::TimeNow(datetime));
                    }
                    Commands::SetTime(time) => {
                        let offset_utc = time.offset();

                        unsafe {
                            UTC_OFFSET = Some(time.offset());
                        }

                        timezone = offset_utc;

                        let now = PrimitiveDateTime::new(time.date(), time.time());
                        rtc.set_now_utc(now).unwrap()
                    }
                    Commands::SetTimezone(tz) => {
                        timezone = UtcOffset::from_whole_seconds(tz).unwrap();

                        unsafe {
                            UTC_OFFSET = Some(timezone);
                        }
                    }
                    Commands::PauseRendering => {
                        tx_timer.send(true).unwrap();
                    }
                    Commands::ResumeRendering => {
                        tx_timer.send(false).unwrap();
                    }
                    Commands::StartDeepSleep => {
                        tx_timer.send(true).unwrap();
                        break;
                    }
                    _ => {}
                },
                None => break,
            }
        }

        info!("rtc loop done.")
    }

    async fn run_timer(pause_param: watch::Receiver<bool>, tx: Sender<Commands>) {
        let mut pause = pause_param;

        let mut interval = tokio::time::interval(core::time::Duration::from_secs(1));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut pause_flag = false;

        info!("timer loop started");

        loop {
            select! {
                Ok(_) = pause.changed() => {
                    let val = pause.borrow_and_update();
                    pause_flag = *val;
                }
                _ = interval.tick() => {
                    if pause_flag {
                        info!("pause");
                        continue;
                    }

                    tx.send(Commands::GetTimeNow).await.unwrap();
                }
            }
        }
    }
}
