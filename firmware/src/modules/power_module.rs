use crate::peripherals::adc::AdcDevice;
use crate::peripherals::backlight::Backlight;
use crate::peripherals::hal::PinConfig;
use blinky_shared::domain::WakeupCause;
use esp_idf_hal::adc::ADC1;
use esp_idf_hal::gpio::{AnyIOPin, Gpio36, Level, PinDriver, Pull};
use esp_idf_sys::{
    esp_sleep_ext1_wakeup_mode_t_ESP_EXT1_WAKEUP_ALL_LOW, esp_sleep_source_t_ESP_SLEEP_WAKEUP_ALL,
    esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT0, esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT1,
    esp_sleep_source_t_ESP_SLEEP_WAKEUP_TIMER, esp_sleep_source_t_ESP_SLEEP_WAKEUP_UNDEFINED,
    esp_sleep_wakeup_cause_t, gpio_int_type_t_GPIO_INTR_LOW_LEVEL, gpio_num_t_GPIO_NUM_34,
};
use log::info;
use std::sync::Arc;
use tokio::select;
use tokio::sync::Notify;
use tokio::time::Duration;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;
use blinky_shared::message_bus::{BusHandler, BusSender, MessageBus};

pub struct PowerModule {}

struct Context {
    idle_reset: Arc<Notify>,
}

impl BusHandler<Context> for PowerModule {
    async fn event_handler(bus: &BusSender, context: &mut Context, event: Events) {
        match event {
            Events::TouchOrMove | Events::BleClientConnected => {
                bus.send_cmd(Commands::ResumeRendering);
                context.idle_reset.notify_one();
            }
            _ => {}
        }
    }

    async fn command_handler(_bus: &BusSender, _context: &mut Context, _command: Commands) {}
}

impl PowerModule {
    const TILL_SCREEN_OFF_SEC: u64 = 5;
    const TILL_DEEP_SLEEP_SEC: u64 = 30;
    const TILL_LIGHT_SLEEP_SEC: u64 = 10;

    pub async fn start(adc: ADC1, gpio36: Gpio36, config: PinConfig, bus: MessageBus) {
        info!("starting...");

        Self::announce_wakeup_cause(&bus).await;
        Self::announce_battery_level(&bus, adc, gpio36).await;

        let backlight = Self::init_backlight(config.backlight);
        let idle_reset = Arc::new(Notify::new());

        let idle_scenario = tokio::spawn(Self::idle_sequence(
            bus.clone(),
            backlight,
            idle_reset.clone(),
        ));

        let context = Context { idle_reset };

        MessageBus::handle::<Context, Self>(bus, context).await;

        idle_scenario.await.unwrap();

        info!("done.");
    }

    async fn idle_sequence(bus: MessageBus, mut backlight: Backlight<'_>, token: Arc<Notify>) {
        info!("idle_sequence");
        loop {
            info!("started idle sequence...");
            bus.send_cmd(Commands::ResumeRendering);

            backlight.on();

            if !(Self::try_await_for(Self::TILL_SCREEN_OFF_SEC, &token).await) {
                info!("abort idle sequence on TILL_SCREEN_OFF_SEC");
                continue;
            }

            bus.send_cmd(Commands::PauseRendering);
            backlight.off();

            if !(Self::try_await_for(Self::TILL_LIGHT_SLEEP_SEC, &token).await) {
                info!("abort idle sequence on TILL_LIGHT_SLEEP_SEC");
                backlight.on();
                continue;
            }

            Self::goto_light_sleep();
            let wakeup_cause = Self::get_wakeup_cause().await;

            info!("after light sleep, wakeup_cause {:?}", wakeup_cause);

            if wakeup_cause == WakeupCause::Timer {
                bus.send_cmd(Commands::StartDeepSleep);
                break;
            }

            bus.send_event(Events::Wakeup(wakeup_cause));
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

    async fn get_wakeup_cause() -> WakeupCause {
        let esp_cause = Self::get_wakeup_cause_esp().await;
        let cause = match esp_cause {
            esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT0 => WakeupCause::Ext0,
            esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT1 => WakeupCause::Ext1,
            esp_sleep_source_t_ESP_SLEEP_WAKEUP_UNDEFINED => WakeupCause::Undef,
            esp_sleep_source_t_ESP_SLEEP_WAKEUP_TIMER => WakeupCause::Timer,
            esp_sleep_source_t_ESP_SLEEP_WAKEUP_ULP => WakeupCause::Ulp,
        };

        return cause;
    }

    async fn get_wakeup_cause_esp() -> esp_sleep_wakeup_cause_t {
        let result = tokio::task::spawn_blocking(|| unsafe {
            let cause = esp_idf_sys::esp_sleep_get_wakeup_cause();

            let ext1 = esp_idf_sys::esp_sleep_get_ext1_wakeup_status();
            let touch = esp_idf_sys::esp_sleep_get_touchpad_wakeup_status();

            info!("wakeup debug, ext1 {:?} touch {:?}", ext1, touch);

            cause
        })
        .await;

        return result.unwrap();
    }

    fn setup_wakeup_sources() {
        unsafe {
            let _result = esp_idf_sys::esp_sleep_enable_ext0_wakeup(gpio_num_t_GPIO_NUM_34, 0); // key 2

            let _result_ext1 = esp_idf_sys::esp_sleep_enable_ext1_wakeup(
                // accel, touchpad
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

    const ADC_MIN: u16 = 1600;
    const ADC_MAX: u16 = 2050;

    fn convert_to_percent(adc_level: u16) -> u16 {
        let percent: u32 =
            100 * ((adc_level - Self::ADC_MIN) as u32) / (Self::ADC_MAX - Self::ADC_MIN) as u32;

        if percent > 100 {
            return 100;
        }

        return percent as u16;
    }

    async fn announce_wakeup_cause(bus: &MessageBus) {
        let wakeup_cause = Self::get_wakeup_cause().await;
        info!("startup wakeup cause {:?}", wakeup_cause);
        bus.send_event(Events::Wakeup(wakeup_cause));
    }

    async fn announce_battery_level(bus: &MessageBus, adc: ADC1, gpio36: Gpio36) {
        let mut adc_device = AdcDevice::new(adc, gpio36);
        let adc_value = adc_device.read();
        info!("adc {:?}", adc_value);

        bus.send_event(Events::BatteryLevel(Self::convert_to_percent(adc_value)));

        let is_charging = Self::is_charging();
        bus.send_event(Events::Charging(is_charging));
    }
}
