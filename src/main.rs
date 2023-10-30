use std::{f32, ptr, thread, time::Duration};
use std::cell::RefCell;
use std::mem::size_of;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_8X13},
        MonoTextStyle,
    },
    pixelcolor::Rgb565,
    prelude::{*, DrawTarget, RgbColor},
    text::Text,
};
use embedded_graphics::mono_font::iso_8859_16::FONT_10X20;
use embedded_svc::storage::RawStorage;
use esp_idf_hal::{delay, delay::Ets, gpio::{AnyIOPin, Gpio13, Gpio14, Gpio15, Gpio19, Gpio27}, gpio::PinDriver, i2c::I2cConfig, i2c::I2cDriver, peripherals::Peripherals, prelude::*, spi::{Dma, SPI2, SpiDeviceDriver, SpiDriver}};

use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use esp_idf_hal::modem::Modem;

use esp_idf_svc::{eventloop::EspSystemEventLoop, netif::{ EspNetif }, nvs, nvs::EspDefaultNvsPartition, ping, sntp, wifi::{ EspWifi }};
use esp_idf_svc::eventloop::{EspEventLoop, System};
use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsDefault};
use esp_idf_svc::sntp::{OperatingMode, SntpConf, SyncMode, SyncStatus};

use esp_idf_sys::{self as _, esp, esp_app_desc, esp_sleep_wakeup_cause_t, EspError, ets_delay_us, gpio_num_t_GPIO_NUM_34, gpio_num_t_GPIO_NUM_35, time_t};
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_sys::{
    esp_sleep_ext1_wakeup_mode_t_ESP_EXT1_WAKEUP_ALL_LOW, esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT0,
    esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT1, esp_sleep_source_t_ESP_SLEEP_WAKEUP_ULP,
    esp_sleep_source_t_ESP_SLEEP_WAKEUP_UNDEFINED,
};
use pcf8563::{DateTime, PCF8563};
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, UtcOffset};
use time::macros::{datetime, format_description, offset};
use bma423::{Bma423, FeatureInterruptStatus, Features, InterruptLine, PowerControlFlag};
use cst816s::{CST816S, TouchEvent};
use embedded_graphics::primitives::{Circle, PrimitiveStyle};
use embedded_svc::io::Write;
use embedded_hal::digital::OutputPin;
use embedded_hal_bus::i2c::RefCellDevice;

use esp_idf_hal::{
    gpio::{self, Output},
    i2c
};
use esp_idf_hal::gpio::{Gpio12, Gpio21, Gpio33, Input, InterruptType, IOPin};
use esp_idf_hal::i2c::I2c;

use esp32_nimble::{uuid128, BLEDevice, NimbleProperties};
use esp_idf_hal::spi::config::DriverConfig;
use mipidsi::Builder;
use embedded_hal_compat::{ForwardCompat, Reverse, ReverseCompat};

use serde::{Deserialize, Serialize};
use tokio::join;
use crate::error::Error;
use crate::peripherals::bluetooth::{Bluetooth, BluetoothConfig};

use tokio::sync::broadcast;

use esp_idf_svc::log::{EspLogger, set_target_level};
use log::*;
use crate::modules::accel_module::AccelerometerModule;
use crate::modules::ble_module::BleModule;
use crate::modules::power_module::PowerModule;
use crate::modules::reference_time::{GpsCoordinates, ReferenceTime};

use crate::modules::renderer::Renderer;
use crate::modules::rtc_module::RtcModule;

use crate::modules::time_sync::{RtcSync};
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
    pub mod bluetooth;
    pub mod wifi;
    pub mod nvs_storage;
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

use crate::peripherals::bma423ex::{AxesConfig, Bma423Ex, InterruptIOCtlFlags};
use crate::peripherals::display::ClockDisplay;
use crate::peripherals::hal::{HAL, Commands, HalConfig, Events, PinConfig};
use crate::peripherals::i2c_proxy_async::I2cProxyAsync;
use crate::peripherals::touchpad::TouchpadConfig;
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
    pub departureTime: i64,
    pub arrivalTime: i64,

    pub rideType: RideType,
    pub delayMinutes: i32,
    pub route: String,
    pub from: String,
    pub to: String
}

#[link_section = ".rtc.data"]
static mut CurrentCoords: GpsCoordinates = GpsCoordinates {lat: 0.0, lon: 0.0};

esp_app_desc!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    EspLogger::initialize_default();
    ::log::set_max_level(LevelFilter::Trace);

    /*
    info!("Setting up eventfd...");
    // eventfd is needed by our mio poll implementation.  Note you should set max_fds
    // higher if you have other code that may need eventfd.
    let config = esp_idf_sys::esp_vfs_eventfd_config_t {
        max_fds: 1,
        ..Default::default()
    };
    esp! { unsafe { esp_idf_sys::esp_vfs_eventfd_register(&config) } }?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()?;
    */

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .worker_threads(2)
        .thread_stack_size(50 * 1024)
        .build()?;

    //let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        main_async().await
    })
}

async fn main_async() -> Result<(), Box<dyn std::error::Error>> {
    info!("main_async...");

    let peripherals = Peripherals::take().unwrap();

    let hal_conf = HalConfig {
        backlight: 21,
        touch_interrupt_pin: 12,
        touch_reset_pin: 33,
        ble_config: BluetoothConfig { },
        wifi_config: WifiConfig {
            is_enabled: false,
            ssid: String::from(SSID),
            pass: String::from(PASS)
        }
    };

    let (commands_sender, _) = broadcast::channel::<Commands>(16);
    let (events_sender, _) = broadcast::channel::<Events>(16);

    let mut hal= HAL::new(hal_conf, peripherals);

    let i2c_proxy_async = hal.get_i2c_proxy_async().clone();

    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let rtc_task = tokio::spawn(async move {
        RtcModule::start(i2c_proxy_async, commands_channel, events_channel).await;
    });

    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let time_sync_task = tokio::spawn(async move {
        RtcSync::start(commands_channel, events_channel).await;
    });

    let ble_config = hal.config.ble_config;
    let commands_channel = commands_sender.clone();
    let events_channel = events_sender.clone();

    let ble_task = tokio::spawn(async move {
        BleModule::start(ble_config, commands_channel, events_channel).await;
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
        PowerModule::start( pin_conf, commands_channel, events_channel).await;
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

    println!("before join");

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

    println!("done.");

    Ok(())
}