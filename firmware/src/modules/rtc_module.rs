use std::collections::BTreeSet;

use blinky_shared::reminders::Reminder;
use log::info;
use time::{PrimitiveDateTime, UtcOffset};
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver, Sender};
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

        let bus_clone = bus.clone();

        let rtc_task = tokio::task::spawn_blocking(move || {
            Self::rtc_loop(bus_clone, rx_rtc, rtc);
        });

        let timer = tokio::spawn(Self::run_timer(tx_rtc.clone()));

        let context = Context { tx_rtc };

        MessageBus::handle::<Context, Self>(bus, context).await;

        rtc_task.await.unwrap();

        timer.abort();

        info!("done.");
    }

    fn rtc_loop(bus: MessageBus, mut rx: Receiver<Commands>, rtc_param: Rtc) {
        let mut reminders: BTreeSet<Reminder> = BTreeSet::new();

        let mut timezone: UtcOffset = Self::get_timezone();

        let mut rtc = rtc_param;

        let datetime = rtc.get_now_utc().assume_offset(timezone);
        bus.send_event(Events::TimeNow(datetime));

        let mut is_paused = false;

        loop {
            let command_opt = rx.blocking_recv();

            match command_opt {
                Some(command) => match command {
                    Commands::GetTimeNow => {
                        let now = rtc.get_now_utc().assume_offset(timezone);

                        if !is_paused {
                            bus.send_event(Events::TimeNow(now));
                        }

                        invoke_reminders(&mut reminders, now, &bus);
                    }
                    Commands::SetReminders(mut reminders_param) => {
                        for reminder in reminders_param.drain(..) {
                            info!("set reminder {:?}", reminder);
                            reminders.insert(reminder);
                        }
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
                    Commands::SetRtcAlert(alert_at) => {
                        rtc.set_alarm(alert_at);

                        info!("set rtc alert for {}", alert_at);
                    }
                    Commands::ResetRtcAlert => rtc.reset_alarm(),
                    Commands::PauseRendering => {
                        is_paused = true;
                    }
                    Commands::ResumeRendering => {
                        is_paused = false;
                    }
                    Commands::StartDeepSleep => {
                        is_paused = false;
                        break;
                    }
                    _ => {}
                },
                None => break,
            }
        }

        info!("rtc loop done.")
    }

    async fn run_timer(tx: Sender<Commands>) {
        let mut interval = tokio::time::interval(core::time::Duration::from_secs(1));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        info!("timer loop started");

        loop {
            select! {
                _ = interval.tick() => {
                    tx.send(Commands::GetTimeNow).await.unwrap();
                }
            }
        }
    }

    fn get_timezone() -> UtcOffset {
        let utc_offset = unsafe {
            if UTC_OFFSET.is_none() {
                UtcOffset::from_whole_seconds(0).unwrap()
            } else {
                UTC_OFFSET.unwrap()
            }
        };
        utc_offset
    }
}

fn invoke_reminders(
    reminders: &mut BTreeSet<Reminder>,
    now: time::OffsetDateTime,
    bus: &MessageBus,
) {
    loop {
        let first_opt = reminders.first();

        if first_opt.is_none() {
            break;
        }

        let remind_at = first_opt.unwrap().remind_at;
        if now < remind_at {
            break;
        }

        let first = reminders.pop_first();

        if now >= remind_at {
            bus.send_event(Events::Reminder(first.unwrap()));
        }
    }
}
