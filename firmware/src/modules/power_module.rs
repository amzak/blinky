use crate::peripherals::adc::AdcDevice;
use crate::peripherals::hal::PinConfig;
use crate::peripherals::output::PinOutput;
use blinky_shared::domain::WakeupCause;
use blinky_shared::reminders::ReminderKind;
use esp_idf_hal::adc::Adc;
use esp_idf_hal::gpio::{ADCPin, AnyIOPin, Level, Output, OutputPin, Pin, PinDriver, Pull};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_sys::{
    esp_sleep_ext1_wakeup_mode_t_ESP_EXT1_WAKEUP_ALL_LOW, esp_sleep_source_t_ESP_SLEEP_WAKEUP_ALL,
    esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT0, esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT1,
    esp_sleep_source_t_ESP_SLEEP_WAKEUP_TIMER, esp_sleep_source_t_ESP_SLEEP_WAKEUP_UNDEFINED,
    esp_sleep_wakeup_cause_t, gpio_int_type_t_GPIO_INTR_LOW_LEVEL, gpio_num_t_GPIO_NUM_34,
};
use log::info;
use peripherals::pins::mapping::PinsMapping;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tokio::select;
use tokio::sync::Notify;
use tokio::time::{sleep, Duration};

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;
use blinky_shared::message_bus::{BusHandler, BusSender, MessageBus};

pub struct PowerModule {}

struct Context {
    idle_reset: Arc<Notify>,
    config: PinConfig,
}

impl BusHandler<Context> for PowerModule {
    async fn event_handler(bus: &BusSender, context: &mut Context, event: Events) {
        match event {
            Events::SharedInterrupt
            | Events::Key1Press
            | Events::Key2Press
            | Events::BleClientConnected => {
                bus.send_cmd(Commands::ResumeRendering);
                context.idle_reset.notify_one();
            }
            Events::Reminder(reminder) => {
                context.idle_reset.notify_one();

                if matches!(reminder.kind, ReminderKind::Notification) {
                    Self::signal_reminder(&context.config, 2).await;
                } else {
                    Self::signal_reminder(&context.config, 3).await;
                }
            }
            _ => {}
        }
    }

    async fn command_handler(_bus: &BusSender, _context: &mut Context, _command: Commands) {}
}

impl PowerModule {
    const TILL_SCREEN_OFF_SEC: u64 = 10;
    const TILL_DEEP_SLEEP_SEC: u64 = 30;
    const TILL_LIGHT_SLEEP_SEC: u64 = 10;

    fn setup_wakeup_sources(pins_mapping: Arc<Mutex<impl PinsMapping>>) {
        unsafe {
            let button1_pin = pins_mapping.lock().unwrap().get_button1_pin_index();

            let _result = esp_idf_sys::esp_sleep_enable_ext0_wakeup(button1_pin, 0); // key 2

            let touch_int_pin = pins_mapping.lock().unwrap().get_touch_int_pin_index();

            let _result_ext1 = esp_idf_sys::esp_sleep_enable_ext1_wakeup(
                // accel, touchpad
                1 << touch_int_pin,
                esp_sleep_ext1_wakeup_mode_t_ESP_EXT1_WAKEUP_ALL_LOW,
            );
        }
    }

    pub async fn start<TAdc, TAdcPin, TBacklightPin, PM>(
        adc: impl Peripheral<P = TAdc>,
        pins_mapping: Arc<Mutex<PM>>,
        backlight: Option<PinDriver<'_, TBacklightPin, Output>>,
        config: PinConfig,
        bus: MessageBus,
    ) where
        TAdc: Adc,
        TAdcPin: ADCPin<Adc = TAdc>,
        TBacklightPin: OutputPin,
        PM: PinsMapping<TAdcPin = TAdcPin, TBacklightPin = TBacklightPin>,
    {
        info!("starting...");

        if backlight.is_some() {
            let mut backlight = backlight.unwrap();
            backlight.set_high().unwrap();
        }
        //let backlight = Self::init_backlight(backlight.pin());

        let idle_reset = Arc::new(Notify::new());

        let idle_scenario = tokio::spawn(Self::idle_sequence(bus.clone(), idle_reset.clone()));

        let wakeup_cause = Self::get_wakeup_cause().await;
        Self::announce_wakeup_cause(&bus, &wakeup_cause);

        if matches!(wakeup_cause, WakeupCause::Undef) {
            Self::signal_reminder(&config, 2).await;
        }

        let adc_pin = pins_mapping.lock().unwrap().get_adc_pin();

        let mut adc_device = AdcDevice::new(adc, adc_pin);

        Self::announce_battery_level(&bus, &mut adc_device);
        tokio::time::sleep(Duration::from_secs(1)).await;
        Self::announce_battery_level(&bus, &mut adc_device);

        let context = Context { idle_reset, config };

        MessageBus::handle::<Context, Self>(bus, context).await;

        idle_scenario.await.unwrap();

        Self::setup_wakeup_sources(pins_mapping.clone());

        info!("done.");
    }

    async fn idle_sequence(bus: MessageBus, /*backlight: PinOutput<'_>,*/ token: Arc<Notify>) {
        info!("idle_sequence");

        //let backlight = backlight

        loop {
            info!("started idle sequence...");
            bus.send_cmd(Commands::ResumeRendering);

            // backlight.on();

            if !(Self::try_await_for(Self::TILL_SCREEN_OFF_SEC, &token).await) {
                info!("abort idle sequence on TILL_SCREEN_OFF_SEC");
                continue;
            }

            bus.send_cmd(Commands::PauseRendering);
            // backlight.off();

            if !(Self::try_await_for(Self::TILL_LIGHT_SLEEP_SEC, &token).await) {
                info!("abort idle sequence on TILL_LIGHT_SLEEP_SEC");
                // backlight.on();
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

    fn init_backlight(backlight_pin: i32) -> PinOutput<'static> {
        let backlight = PinOutput::create(backlight_pin, true);

        backlight
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

    pub fn goto_deep_sleep() {
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
            esp_idf_sys::esp_sleep_enable_timer_wakeup(Self::TILL_DEEP_SLEEP_SEC as u64 * 1000_000);
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
        return 50;

        let percent: i32 =
            100 * ((adc_level - Self::ADC_MIN) as i32) / (Self::ADC_MAX - Self::ADC_MIN) as i32;

        if percent > 100 {
            return 100;
        }

        return percent as u16;
    }

    fn announce_wakeup_cause(bus: &MessageBus, wakeup_cause: &WakeupCause) {
        info!("startup wakeup cause {:?}", wakeup_cause);
        bus.send_event(Events::Wakeup(wakeup_cause.clone()));
    }

    fn announce_battery_level<TAdcPin: ADCPin>(bus: &MessageBus, adc: &mut AdcDevice<TAdcPin>) {
        let adc_value = adc.read();

        let is_charging = Self::is_charging();

        if is_charging {
            bus.send_event(Events::Charging(is_charging));
        } else {
            bus.send_event(Events::BatteryLevel(Self::convert_to_percent(adc_value)));
        }
    }

    async fn signal_reminder(config: &PinConfig, count: i8) {
        let mut vibro = PinOutput::create(config.vibro, true);

        for i in 0..count - 1 {
            if i != 0 {
                sleep(Duration::from_millis(300)).await;
                vibro.on();
            }
            sleep(Duration::from_millis(400)).await;
            vibro.off();
        }
    }
}
