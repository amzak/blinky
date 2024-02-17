#![feature(slice_as_chunks)]
#![feature(vec_push_within_capacity)]
#![feature(associated_type_bounds)]

use blinky_shared::commands::Commands;
use blinky_shared::message_bus::MessageBus;
use blinky_shared::modules::renderer::Renderer;
use esp_idf_hal::i2c::{I2cDriver, I2C0};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::log::{set_target_level, EspLogger};
use log::*;
use peripherals::i2c_proxy_async::I2cProxyAsync;
use std::future::Future;
use std::thread;
use tokio::join;
use tokio::task::JoinHandle;

extern crate blinky_shared;

mod modules;
mod peripherals;

use peripherals::hal::{HalConfig, PinConfig, HAL};

use modules::accel_module::AccelerometerModule;
use modules::ble_module::BleModule;
use modules::calendar_module::CalendarModule;
use modules::persister_module::PersisterModule;
use modules::power_module::PowerModule;
use modules::reference_time::ReferenceTime;
use modules::rtc_module::RtcModule;
use modules::time_sync::TimeSync;
use modules::touch_module::TouchModule;
use modules::user_input::UserInput;

use crate::modules::logging_module::LoggingModule;
use crate::peripherals::display::ClockDisplay;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    EspLogger::initialize_default();
    set_max_level(LevelFilter::Info);
    set_target_level("spi_master", LevelFilter::Error).unwrap();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .worker_threads(2)
        .on_thread_start(|| {
            let core = esp_idf_hal::cpu::core();

            info!(
                "thread started {:?} {:?} core {:?}",
                thread::current().id(),
                thread::current().name(),
                core
            );
        })
        .thread_stack_size(10 * 1024)
        .build()?;

    rt.block_on(main_async())?;

    PowerModule::goto_deep_sleep();

    Ok(())
}

async fn main_async() -> Result<(), Box<dyn std::error::Error>> {
    info!("main_async...");

    let peripherals = Peripherals::take().unwrap();

    let hal_conf = HalConfig {
        backlight: 21,
        touch_interrupt_pin: 12,
        touch_reset_pin: 33,
    };

    let message_bus = MessageBus::new();

    let hal: HAL = HAL::new(hal_conf, peripherals.i2c0);

    let logging_task = start_logging(&message_bus);

    let i2c_proxy = hal.get_i2c_proxy_async().clone();
    let rtc_task = start_rtc(&message_bus, i2c_proxy);

    let time_sync_task = start_time_sync(&message_bus);

    let renderer_task = start_renderer(&message_bus);

    let ble_task = start_ble(&message_bus);

    let pin_conf = PinConfig { backlight: 21 };
    let mb = message_bus.clone();
    let power_task = PowerModule::start(peripherals.adc1, peripherals.pins.gpio36, pin_conf, mb);

    let mb = message_bus.clone();
    let user_input_task = UserInput::start(mb);

    let mb = message_bus.clone();
    let touch_config = hal.get_touch_config();
    let touch_proxy = hal.get_i2c_proxy_async();
    let touch_task = TouchModule::start(touch_config, touch_proxy, mb);

    let accel_proxy = hal.get_i2c_proxy_async();
    let accel_proxy_ex = hal.get_i2c_proxy_async();

    let mb = message_bus.clone();
    let accel_task = AccelerometerModule::start(accel_proxy, accel_proxy_ex, mb);

    let mb = message_bus.clone();
    let reference_time_task = ReferenceTime::start(mb);

    let mb = message_bus.clone();
    let calendar_task = CalendarModule::start(mb);

    let mb = message_bus.clone();
    let persister_task = PersisterModule::start(mb);

    let startup_sequence = async move {
        message_bus.send_cmd(Commands::SyncRtc);
        message_bus.send_cmd(Commands::SyncCalendar);
        message_bus.send_cmd(Commands::GetTemperature);
    };

    let res = join!(
        logging_task,
        rtc_task,
        time_sync_task,
        ble_task,
        renderer_task,
        user_input_task,
        touch_task,
        accel_task,
        reference_time_task,
        power_task,
        persister_task,
        calendar_task,
        startup_sequence,
    );

    info!("done.");

    Ok(())
}

#[inline]
fn start_logging(mb: &MessageBus) -> impl Future<Output = ()> {
    let mb = mb.clone();
    LoggingModule::start(mb)
}

fn start_time_sync(mb: &MessageBus) -> impl Future<Output = ()> {
    let mb = mb.clone();
    TimeSync::start(mb)
}

fn start_rtc(
    mb: &MessageBus,
    i2c_proxy: I2cProxyAsync<I2cDriver<'static>>,
) -> impl Future<Output = ()> {
    let mb = mb.clone();
    RtcModule::start(i2c_proxy, mb)
}

fn start_ble(mb: &MessageBus) -> impl Future<Output = ()> {
    let mb = mb.clone();
    BleModule::start(mb)
}

fn start_renderer(mb: &MessageBus) -> impl Future<Output = ()> {
    let mb = mb.clone();
    Renderer::<ClockDisplay>::start(mb)
}
