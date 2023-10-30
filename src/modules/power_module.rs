use esp_idf_sys::{esp_sleep_ext1_wakeup_mode_t_ESP_EXT1_WAKEUP_ALL_LOW, esp_sleep_wakeup_cause_t, gpio_num_t_GPIO_NUM_34};
use tokio::sync::broadcast::{Sender, Receiver};
use crate::peripherals::hal::{Commands, Events, PinConfig, WakeupCause};
use std::time::Duration;
use esp_idf_hal::gpio::{AnyIOPin, Output, PinDriver};
use log::info;
use esp_idf_svc::timer;
use crate::peripherals::backlight::Backlight;

pub struct PowerModule {

}

impl PowerModule {
    const SCREEN_OFF_SEC: f32 = 4.0;
    const DEEP_SLEEP_SEC: f32 = 30.0;

    pub async fn start(config: PinConfig, commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let wakeup_cause = Self::get_wakeup_cause();
        events.send(Events::WakeupCause(wakeup_cause)).unwrap();

        let mut backlight = Self::init_backlight(config.backlight);

        let timer_service = timer::EspTaskTimerService::new().unwrap();

        let ev1 = events.clone();
        let pwr_timer = timer_service.timer(move || {
            ev1.send(Events::PowerDownTimer).unwrap();
        }).unwrap();

        let scr_off_timer = timer_service.timer(move || {
            events.send(Events::ScreenOffTimer).unwrap();
        }).unwrap();

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    match command {
                        _ => {}
                    }
                },
                Ok(event) = recv_event.recv() => {
                    info!("{:?}", event);
                    match event {
                        Events::WakeupCause(cause) => {
                            backlight.on();
                            scr_off_timer.after(Duration::from_secs_f32(Self::SCREEN_OFF_SEC)).unwrap();
                        }
                        Events::TimeNow(now) => {
                        }
                        Events::TouchOrMove => {
                            if !backlight.is_on() {
                                backlight.on();
                                scr_off_timer.cancel().unwrap();
                                pwr_timer.cancel().unwrap();
                            }

                            scr_off_timer.after(Duration::from_secs_f32(Self::SCREEN_OFF_SEC)).unwrap();
                        }
                        Events::ScreenOffTimer => {
                            println!("{:?}", event);
                            backlight.off();
                            pwr_timer.after(Duration::from_secs_f32(Self::DEEP_SLEEP_SEC));
                        }
                        Events::PowerDownTimer => {
                            println!("{:?}", event);
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        Self::goto_deep_sleep();
    }

    fn init_backlight(backlight_pin: i32) -> Backlight<'static> {
        Backlight::create(backlight_pin)
    }

    fn get_wakeup_cause() -> WakeupCause {
        let esp_cause = Self::get_wakeup_cause_esp();
        let cause = match esp_cause {
            esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT0 => WakeupCause::Ext0,
            esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT1 => WakeupCause::Ext1,
            esp_sleep_source_t_ESP_SLEEP_WAKEUP_UNDEFINED => WakeupCause::Undef,
            esp_sleep_source_t_ESP_SLEEP_WAKEUP_TIMER => WakeupCause::Timer,
            esp_sleep_source_t_ESP_SLEEP_WAKEUP_ULP => WakeupCause::Ulp
        };

        return cause;
    }

    fn get_wakeup_cause_esp() -> esp_sleep_wakeup_cause_t {
        unsafe {
            return esp_idf_sys::esp_sleep_get_wakeup_cause();
        }
    }

    fn goto_deep_sleep() {
        unsafe {
            let result = esp_idf_sys::esp_sleep_enable_ext0_wakeup(gpio_num_t_GPIO_NUM_34, 0);
            println!("esp_sleep_enable_ext0_wakeup result {}", result);

            let result_ext1 = esp_idf_sys::esp_sleep_enable_ext1_wakeup(
                1 << 32,
                esp_sleep_ext1_wakeup_mode_t_ESP_EXT1_WAKEUP_ALL_LOW,
            );
            println!("esp_sleep_enable_ext1_wakeup result {}", result_ext1);

            println!("going to deep sleep");
            esp_idf_sys::esp_deep_sleep_disable_rom_logging();
            esp_idf_sys::esp_deep_sleep_start();
        }
    }
}