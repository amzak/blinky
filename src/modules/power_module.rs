use std::sync::Arc;
use esp_idf_hal::adc::ADC1;
use esp_idf_hal::gpio::{AnyIOPin, Gpio36, Level, PinDriver, Pull};
use esp_idf_sys::{esp_sleep_ext1_wakeup_mode_t_ESP_EXT1_WAKEUP_ALL_LOW, esp_sleep_source_t_ESP_SLEEP_WAKEUP_ALL, esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT0, esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT1, esp_sleep_source_t_ESP_SLEEP_WAKEUP_TIMER, esp_sleep_source_t_ESP_SLEEP_WAKEUP_UNDEFINED, esp_sleep_wakeup_cause_t, gpio_int_type_t_GPIO_INTR_LOW_LEVEL, gpio_num_t_GPIO_NUM_34};
use tokio::sync::broadcast::Sender;
use crate::peripherals::hal::{Commands, Events, PinConfig, WakeupCause};
use log::info;
use tokio::select;
use tokio::sync::Notify;
use crate::peripherals::backlight::Backlight;
use tokio::time::Duration;
use crate::peripherals::adc::AdcDevice;

pub struct PowerModule {

}

#[repr(u8)]
pub enum PowerMode {
    On,
    ScreenOff,
    LightSleep,
    DeepSleep
}

impl PowerModule {
    const TILL_SCREEN_OFF_SEC: u64 = 4;
    const TILL_DEEP_SLEEP_SEC: u64 = 30;
    const TILL_LIGHT_SLEEP_SEC: u64 = 5;

    pub async fn start(adc: ADC1, gpio36: Gpio36, config: PinConfig, commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let wakeup_cause = Self::get_wakeup_cause();
        info!("startup wakeup cause {:?}", wakeup_cause);

        let mut backlight = Self::init_backlight(config.backlight);

        let mut adc_device = AdcDevice::new(adc, gpio36);
        let adc_value = adc_device.read();
        info!("adc {:?}", adc_value);
        events.send(Events::BatteryLevel(adc_value)).unwrap();

        let is_charging = Self::is_charging();
        events.send(Events::Charging(is_charging)).unwrap();

        let cm1 = commands.clone();
        let ev1 = events.clone();

        let reset_idle = Arc::new(Notify::new());

        let idle_scenario = tokio::spawn(Self::idle_sequence(ev1, cm1, reset_idle.clone()));

        events.send(Events::Wakeup(wakeup_cause)).unwrap();

        loop {
            select! {
                Ok(command) = recv_cmd.recv() => {
                    info!("{:?}", command);
                    match command {
                        Commands::PauseRendering => {
                            backlight.off();
                        }
                        Commands::ResumeRendering => {
                            backlight.on();
                            reset_idle.notify_one();
                        }
                        Commands::StartDeepSleep => {
                            break;
                        }
                        _ => {}
                    }
                },
                Ok(event) = recv_event.recv() => {
                    info!("{:?}", event);
                    match event {
                        Events::Wakeup(_) => {
                            commands.send(Commands::ResumeRendering).unwrap();
                        }
                        Events::TouchOrMove => {
                            commands.send(Commands::ResumeRendering).unwrap();
                        }
                        _ => {}
                    }
                }
            }
        }

        idle_scenario.await.unwrap();

        info!("done.");
    }

    async fn idle_sequence(events: Sender<Events>, commands: Sender<Commands>, token: Arc<Notify>) {
        info!("idle_sequence");

        loop {
            info!("started idle sequence...");

            if !(Self::try_await_for(Self::TILL_SCREEN_OFF_SEC, &token).await) {
                info!("abort idle sequence on TILL_SCREEN_OFF_SEC");
                continue;
            }

            commands.send(Commands::PauseRendering).unwrap();

            if !(Self::try_await_for(Self::TILL_LIGHT_SLEEP_SEC, &token).await) {
                info!("abort idle sequence on TILL_LIGHT_SLEEP_SEC");
                continue;
            }

            Self::goto_light_sleep();
            let wakeup_cause = Self::get_wakeup_cause();

            info!("after light sleep, wakeup_cause {:?}", wakeup_cause);

            if wakeup_cause == WakeupCause::Timer {
                info!("before send StartDeepSleep");
                commands.send(Commands::StartDeepSleep).unwrap();
                info!("after send StartDeepSleep");
                break;
            }

            events.send(Events::Wakeup(wakeup_cause)).unwrap();
        }
    }

    async fn try_await_for(timeout: u64, cancellation: &Notify) -> bool {
        select! {
            _ = tokio::time::sleep(Duration::from_secs(timeout)) => { true }
            _ = cancellation.notified() => { false }
        }
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
        let mut cause = esp_sleep_source_t_ESP_SLEEP_WAKEUP_UNDEFINED;
        unsafe {
            cause = esp_idf_sys::esp_sleep_get_wakeup_cause();

            let ext1 = esp_idf_sys::esp_sleep_get_ext1_wakeup_status();
            let touch = esp_idf_sys::esp_sleep_get_touchpad_wakeup_status();

            info!("wakeup debug, ext1 {:?} touch {:?}", ext1, touch);
        }

        return cause;
    }

    fn setup_wakeup_sources() {
        unsafe {
            let result = esp_idf_sys::esp_sleep_enable_ext0_wakeup(gpio_num_t_GPIO_NUM_34, 0); // key 2

            let result_ext1 = esp_idf_sys::esp_sleep_enable_ext1_wakeup( // accel, touchpad
                                                                         1 << 32,
                                                                         esp_sleep_ext1_wakeup_mode_t_ESP_EXT1_WAKEUP_ALL_LOW,
            );
        }
    }

    pub fn goto_deep_sleep() {
        info!("preparing for deep sleep...");

        Self::setup_wakeup_sources();

        info!("going to deep sleep...");

        unsafe {
            esp_idf_sys::esp_deep_sleep_start();
        }
    }

    fn cleanup_wakeup_sources() {
        unsafe {
            esp_idf_sys::esp_sleep_disable_wakeup_source(esp_sleep_source_t_ESP_SLEEP_WAKEUP_ALL);
        }
    }

    fn goto_light_sleep() {
        info!("going to light sleep...");
        log::logger().flush();

        unsafe {
            esp_idf_sys::gpio_wakeup_enable(32, gpio_int_type_t_GPIO_INTR_LOW_LEVEL);
            esp_idf_sys::esp_sleep_enable_gpio_wakeup();
            esp_idf_sys::esp_sleep_enable_timer_wakeup(Self::TILL_DEEP_SLEEP_SEC as u64 * 1000000);
            esp_idf_sys::esp_light_sleep_start();
        }

        Self::cleanup_wakeup_sources();
    }
    fn is_charging() -> bool {
        let pin = unsafe { AnyIOPin::new(2) };
        let mut pin_driver = PinDriver::input(pin).unwrap();
        pin_driver.set_pull(Pull::Up).unwrap();
        pin_driver.get_level() == Level::Low
    }
}