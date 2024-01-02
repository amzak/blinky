#![feature(slice_as_chunks)]
#![feature(vec_push_within_capacity)]
#![feature(async_fn_in_trait)]

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::log::{set_target_level, EspLogger};
use log::*;
use std::thread;
use tokio::join;
use tokio::sync::broadcast::{self};

mod error;
mod modules;
mod peripherals;
mod persistence;

use peripherals::hal::{Commands, Events, HalConfig, PinConfig, HAL};

use modules::accel_module::AccelerometerModule;
use modules::ble_module::BleModule;
use modules::calendar_module::CalendarModule;
use modules::persister_module::PersisterModule;
use modules::power_module::PowerModule;
use modules::reference_time::ReferenceTime;
use modules::renderer::Renderer;
use modules::rtc_module::RtcModule;
use modules::time_sync::TimeSync;
use modules::touch_module::TouchModule;
use modules::user_input::UserInput;

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
                "thread started {:?} core {:?}",
                thread::current().id(),
                core
            );
        })
        .thread_stack_size(30 * 1024)
        .build()?;

    rt.block_on(async { main_async().await })?;

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

    let (commands_sender, _) = broadcast::channel::<Commands>(32);
    let (events_sender, _) = broadcast::channel::<Events>(32);

    let mut hal = HAL::new(hal_conf, peripherals.i2c0);

    let i2c_proxy_async = hal.get_i2c_proxy_async().clone();

    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let rtc_task = tokio::spawn(async move {
        RtcModule::start(i2c_proxy_async, commands_channel, events_channel).await;
    });

    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let time_sync_task = tokio::spawn(async move {
        TimeSync::start(commands_channel, events_channel).await;
    });

    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let ble_task = tokio::spawn(async move {
        BleModule::start(commands_channel, events_channel).await;
    });

    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let renderer_task = tokio::spawn(async move {
        Renderer::start(commands_channel, events_channel).await;
    });

    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let pin_conf = PinConfig { backlight: 21 };

    let power_task = tokio::spawn(async move {
        PowerModule::start(
            peripherals.adc1,
            peripherals.pins.gpio36,
            pin_conf,
            commands_channel,
            events_channel,
        )
        .await;
    });

    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let user_input_task = tokio::spawn(async move {
        UserInput::start(commands_channel, events_channel).await;
    });

    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let touch_config = hal.get_touch_config();
    let touch_proxy = hal.get_i2c_proxy_async();
    let touch_task = tokio::spawn(async move {
        TouchModule::start(touch_config, touch_proxy, commands_channel, events_channel).await;
    });

    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let accel_proxy = hal.get_i2c_proxy_async();
    let accel_proxy_ex = hal.get_i2c_proxy_async();

    let accel_task = tokio::spawn(async move {
        AccelerometerModule::start(
            accel_proxy,
            accel_proxy_ex,
            commands_channel,
            events_channel,
        )
        .await;
    });

    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let reference_time_task = tokio::spawn(async move {
        ReferenceTime::start(commands_channel, events_channel).await;
    });

    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let calendar_task = tokio::spawn(async move {
        CalendarModule::start(commands_channel, events_channel).await;
    });

    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let persister_task = tokio::spawn(async move {
        PersisterModule::start(commands_channel, events_channel).await;
    });

    let commands_channel = commands_sender.clone();

    let startup_sequence = tokio::spawn(async move {
        commands_channel.send(Commands::SyncRtc).unwrap();
        commands_channel.send(Commands::SyncCalendar).unwrap();
        commands_channel.send(Commands::GetTemperature).unwrap();
    });

    join!(
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
