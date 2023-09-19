use std::{error::Error, f32, ptr, thread, time::Duration};
use std::cell::RefCell;
use std::mem::size_of;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

//use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};
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

use esp_idf_svc::{eventloop::EspSystemEventLoop, log, netif::{ EspNetif }, nvs, nvs::EspDefaultNvsPartition, ping, sntp, wifi::{ EspWifi }};
use esp_idf_svc::eventloop::{EspEventLoop, System};
use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsDefault};
use esp_idf_svc::sntp::{OperatingMode, SntpConf, SyncMode, SyncStatus};

use esp_idf_sys::{self as _, esp_sleep_wakeup_cause_t, EspError, ets_delay_us, gpio_num_t_GPIO_NUM_34, gpio_num_t_GPIO_NUM_35, time_t};
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_sys::{
    esp_sleep_ext1_wakeup_mode_t_ESP_EXT1_WAKEUP_ALL_LOW, esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT0,
    esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT1, esp_sleep_source_t_ESP_SLEEP_WAKEUP_ULP,
    esp_sleep_source_t_ESP_SLEEP_WAKEUP_UNDEFINED,
};
use pcf8563::{DateTime, PCF8563};
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, UtcOffset};
use time::macros::{datetime, format_description, offset};
//use esp_idf_svc::log;
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
use esp_idf_hal::gpio::{Gpio12, Gpio21, Gpio33, Input, IOPin};
use esp_idf_hal::i2c::I2c;
use esp_idf_svc::log::EspLogger;

use esp32_nimble::{uuid128, BLEDevice, NimbleProperties};
use esp_idf_hal::spi::config::DriverConfig;
use mipidsi::Builder;
use embedded_hal_compat::{ForwardCompat, Reverse, ReverseCompat};

use serde::{Deserialize, Serialize};
use crate::peripherals::bluetooth::BluetoothConfig;

mod peripherals {
    pub mod bma423ex;
    pub mod backlight;
    pub mod hal;
    pub mod display;
    pub mod accelerometer;
    pub mod i2c_management;
    pub mod i2c_proxy;
    pub mod touchpad;
    pub mod rtc;
    pub mod bluetooth;
}


use crate::peripherals::bma423ex::{AxesConfig, Bma423Ex, InterruptIOCtlFlags};
use crate::peripherals::display::ClockDisplay;
use crate::peripherals::hal::{Devices, HAL, PinConfig};

const SSID: &str = "HOTBOX-B212";
const PASS: &str = "0534337688";
const LAST_SYNC: &str = "last_sync";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LastSyncInfo {
    pub last_sync: OffsetDateTime,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct GpsCoord {
    pub lat: f32,
    pub lon: f32
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
static mut CurrentCoords: GpsCoord = GpsCoord {lat: 0.0, lon: 0.0};

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    EspLogger::initialize_default();

    let wakeup_cause = get_wakeup_cause();
    let wakeup_cause_str = format_wakeup_cause(wakeup_cause);

    let peripherals = Peripherals::take().unwrap();

    let pin_conf = PinConfig {
        backlight: 21,
        touch_interrupt_pin: 12,
        touch_reset_pin: 33,
        ble_config: BluetoothConfig { }
    };

    let mut hal= HAL::new(pin_conf, peripherals);
    let hal_ref = &hal;
    let devices = Devices::new(hal_ref);

    let backlight = hal.backlight();
    backlight.borrow_mut().on();

    hal.display().borrow_mut().clear();
    hal.display().borrow_mut().text("Hello, world!", Point::new(80, 120));
    hal.display().borrow_mut().text(wakeup_cause_str, Point::new(80, 130));

    let coords = get_current_coords();
    let coords_str = format_current_coords(coords);

    hal.display().borrow_mut().text(&coords_str, Point::new(80, 140));

    let sysloop = EspSystemEventLoop::take().unwrap();

    let nvs_partition = nvs::EspDefaultNvsPartition::take().unwrap();
    //let modem = peripherals.modem;
    //let mut _wifi = setupWifi(sysloop, modem, nvs_partition.clone());

    println!("took nvs partition");
    let mut nvs = nvs::EspNvs::new(nvs_partition, "rtc", true).unwrap();

    println!("reading len...");

    let len_opt = nvs.len(LAST_SYNC).map_err(|_| Box::<dyn Error>::from("nvs len error")).unwrap();

    if let Some(len) = len_opt {
        println!("len = {}", len);
        let mut buffer = vec![0; len];
        nvs.get_raw(LAST_SYNC, &mut buffer[..]).unwrap();
        let last_sync_info: LastSyncInfo = postcard::from_bytes::<LastSyncInfo>(&buffer).unwrap();

        println!("last sync {}", last_sync_info.last_sync);

        /* SYNC
        let diff = datetime - last_sync_info.last_sync;

        if diff.whole_days() > 1 {
            sync_rtc(&mut nvs, &mut rtc);
        }

        SYNC */
    }
    else {
        println!("first sync");
        //sync_rtc(&mut nvs, &mut rtc);
    }

    //_wifi.disconnect().unwrap();

    unsafe {
        let result = esp_idf_sys::esp_sleep_enable_ext0_wakeup(gpio_num_t_GPIO_NUM_34, 0);
        println!("result {}", result);

        let result_ext1 = esp_idf_sys::esp_sleep_enable_ext1_wakeup(
            1 << 32,
            esp_sleep_ext1_wakeup_mode_t_ESP_EXT1_WAKEUP_ALL_LOW,
        );
        println!("result {}", result_ext1);

        thread::sleep(Duration::from_millis(1500));
        hal.display().borrow_mut().clear();

        backlight.borrow_mut().off();

        println!("going to deep sleep");
        esp_idf_sys::esp_deep_sleep_disable_rom_logging();
        esp_idf_sys::esp_deep_sleep_start();
    };
}

fn get_current_coords() -> GpsCoord {
    unsafe {
        return GpsCoord {lat: CurrentCoords.lat, lon: CurrentCoords.lon};
    }
}

fn get_wakeup_cause() -> esp_sleep_wakeup_cause_t {
    unsafe {
        return esp_idf_sys::esp_sleep_get_wakeup_cause();
    }
}

fn format_wakeup_cause(cause: esp_sleep_wakeup_cause_t) -> &'static str {
    let formatted = match cause {
        esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT0 => "ext0",
        esp_sleep_source_t_ESP_SLEEP_WAKEUP_EXT1 => "ext1",
        esp_sleep_source_t_ESP_SLEEP_WAKEUP_UNDEFINED => "undef",
        esp_sleep_source_t_ESP_SLEEP_WAKEUP_TIMER => "timer",
        esp_sleep_source_t_ESP_SLEEP_WAKEUP_ULP => "ulp"
    };

    return formatted;
}

fn format_current_coords(coords: GpsCoord) -> String {
    return format!("lat: {} lon: {}", coords.lat, coords.lon);
}

fn setupWifi(sysloop: EspEventLoop<System>, modem: Modem, nvs_partition: EspNvsPartition<NvsDefault>) -> EspWifi<'static> {
    let mut wifi_driver = EspWifi::new(modem, sysloop.clone(), Some(nvs_partition)).unwrap();

    wifi_driver
        .set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            ..Default::default()
        }))
        .unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();

    println!("after wifi connect");

    while wifi_driver.sta_netif().get_ip_info().unwrap().ip.is_unspecified() {
        thread::sleep(Duration::from_millis(2000));
    }

    println!("ip acquired");

    let ip_info = wifi_driver.sta_netif().get_ip_info().unwrap();
    println!("ip: {}", ip_info.ip);
    println!("dns: {:?}", ip_info.dns);

    return wifi_driver;
}

fn sync_rtc(nvs: &mut EspNvs<NvsDefault>, rtc: &mut PCF8563<Reverse<RefCellDevice<I2cDriver>>>)
//fn sync_rtc<T>(nvs: &mut EspNvs<NvsDefault>, rtc: &mut PCF8563<Reverse<T>>)
{
    println!("syncing...");

    let dt = getSntpNow();

    let time = dt.time();
    let date = dt.date();

    let (year, month, day) = date.to_calendar_date();
    let (hour, minute, sec, _) = time.as_hms_micro();

    println!("{}-{}-{}", year, month, day);
    println!("{}:{}:{}", hour, minute, sec);

    let year_rtc = (year - 2000) as u8;

    let rtc_dt = DateTime {
        day,
        year: year_rtc,
        month: month as u8,
        hours: hour,
        minutes: minute,
        seconds: sec,
        weekday: dt.weekday().number_days_from_monday()
    };

    rtc.set_datetime(&rtc_dt).unwrap();

    let last_sync = LastSyncInfo {
        last_sync: dt
    };

    let buf: Vec<u8> = postcard::to_allocvec(&last_sync).unwrap();
    nvs.set_raw(LAST_SYNC, &buf).unwrap();

    println!("sync performed, len = {}", buf.len());
}

fn getSntpNow() -> OffsetDateTime {
    let sntp = sntp::EspSntp::new_default().unwrap();
    println!("SNTP initializing...");

    while sntp.get_sync_status() != SyncStatus::Completed {
        let status = sntp.get_sync_status();
        println!("{:?}", status);
        thread::sleep(Duration::from_millis(1000));
    }

    println!("SNTP ready.");

    let timer: *mut time_t = ptr::null_mut();

    let mut timestamp = 0;

    unsafe {
        timestamp = esp_idf_sys::time(timer);
    }

    let mut dt = OffsetDateTime::from_unix_timestamp(timestamp as i64)
        .unwrap()
        .to_offset(offset!(+3));
    dt
}
