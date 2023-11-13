#![feature(slice_as_chunks)]
#![feature(vec_push_within_capacity)]

use esp_idf_hal::{peripherals::Peripherals};

use time::OffsetDateTime;

use serde::{Deserialize};
use tokio::join;

use tokio::sync::broadcast;

use esp_idf_svc::log::{EspLogger, set_target_level};
use log::*;
use crate::modules::accel_module::AccelerometerModule;
use crate::modules::ble_module::BleModule;
use crate::modules::power_module::PowerModule;
use crate::modules::reference_time::ReferenceTime;

use crate::modules::renderer::Renderer;
use crate::modules::rtc_module::RtcModule;

use crate::modules::time_sync::{TimeSync};
use crate::modules::touch_module::TouchModule;
use crate::modules::user_input::UserInput;

mod peripherals {
    pub mod bma423ex;
    pub mod backlight;
    pub mod hal;
    pub mod display;
    pub mod accelerometer;
    pub mod i2c_management;
    pub mod i2c_proxy_async;
    pub mod touchpad;
    pub mod rtc;
    pub mod wifi;
    pub mod nvs_storage;
    pub mod adc;
}

mod modules {
    pub mod time_sync;
    pub mod reference_time;
    pub mod reference_data;
    pub mod renderer;
    pub mod power_module;
    pub mod user_input;
    pub mod accel_module;
    pub mod touch_module;
    pub mod rtc_module;
    pub mod ble_module;
}


mod error;

use crate::peripherals::hal::{HAL, Commands, HalConfig, Events, PinConfig};
use crate::peripherals::wifi::WifiConfig;

const SSID: &str = "HOTBOX-B212";
const PASS: &str = "0534337688";
const LAST_SYNC: &str = "last_sync";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LastSyncInfo {
    pub last_sync: OffsetDateTime,
}

#[derive(Debug, Deserialize, PartialEq)]
#[repr(u8)]
pub enum RideType {
    train,
    bus
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Ride {
    pub departure_time: i64,
    pub arrival_time: i64,

    pub ride_type: RideType,
    pub delay_minutes: i32,
    pub route: String,
    pub from: String,
    pub to: String
}

//#[link_section = ".rtc.data"]
//static mut CurrentCoords: GpsCoordinates = GpsCoordinates {lat: 0.0, lon: 0.0};

//esp_app_desc!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    EspLogger::initialize_default();
    set_max_level(LevelFilter::Info);
    set_target_level("spi_master", LevelFilter::Error).unwrap();

    unsafe {
        let is_init = esp_idf_sys::esp_spiram_is_initialized();
        let size = esp_idf_sys::esp_spiram_get_size();
        let chip_size = esp_idf_sys::esp_spiram_get_chip_size();

        info!("{} {} {}", is_init, size, chip_size);
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .worker_threads(2)
        .thread_stack_size(30 * 1024)
        .build()?;

    //let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        main_async().await
    })?;

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
        wifi_config: WifiConfig {
            is_enabled: false,
            ssid: String::from(SSID),
            pass: String::from(PASS)
        }
    };

    let (commands_sender, _) = broadcast::channel::<Commands>(16);
    let (events_sender, _) = broadcast::channel::<Events>(16);

    let mut hal= HAL::new(hal_conf, peripherals.i2c0);

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

    let pin_conf = PinConfig {
        backlight: 21
    };

    let power_task = tokio::spawn(async move {
        PowerModule::start( peripherals.adc1, peripherals.pins.gpio36, pin_conf, commands_channel, events_channel).await;
    });

    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let user_input_task = tokio::spawn(async move {
        UserInput::start( commands_channel, events_channel).await;
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
        AccelerometerModule::start( accel_proxy, accel_proxy_ex, commands_channel, events_channel).await;
    });

    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let reference_time_task = tokio::spawn(async move {
        ReferenceTime::start(commands_channel, events_channel).await;
    });

    let commands_channel = commands_sender.clone();

    let startup_sequence = tokio::spawn(async move {
        commands_channel.send(Commands::SyncRtc).unwrap();
    });

    info!("before join");

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
        startup_sequence
    );

    info!("done.");

    Ok(())
}