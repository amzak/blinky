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

mod peripherals {
    //pub mod gc9a01;
    pub mod bma423ex;
    pub mod backlight;
    pub mod hal;
    pub mod display;
    pub mod accelerometer;
    pub mod i2c_management;
    pub mod i2c_proxy;
    //pub mod cst816s;
    //pub mod pin_wrapper;
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
    };

    let mut hal= HAL::new(pin_conf, peripherals);
    //let hal_ref = &hal;
    //let devices = Devices::new(hal_ref);

    let backlight = hal.backlight();
    backlight.borrow_mut().on();

    hal.display().borrow_mut().clear();
    hal.display().borrow_mut().text("Hello, world!", Point::new(80, 120));
    hal.display().borrow_mut().text(wakeup_cause_str, Point::new(80, 130));

    let coords = get_current_coords();
    let coords_str = format_current_coords(coords);

    hal.display().borrow_mut().text(&coords_str, Point::new(80, 140));
    /*
    let scl = peripherals.pins.gpio25;
    let sda = peripherals.pins.gpio26;
    let config = I2cConfig::new().baudrate(100.kHz().into());

    let i2c_driver = I2cDriver::new(peripherals.i2c0, sda, scl, &config).unwrap();

    let i2c_ref_cell = RefCell::new(i2c_driver);

    let proxy_accel = RefCellDevice::new(&i2c_ref_cell);
    let mut accel = Bma423::new_with_address(proxy_accel.reverse(), 0x18);

    let proxy_accel_ex = RefCellDevice::new(&i2c_ref_cell);
    let mut accel_ex = Bma423Ex::new(proxy_accel_ex);

    let interrupt_status = accel.read_interrupt_status().unwrap();
    let feature_interrupt_status: u8 = interrupt_status.feature.into();
    let interrupt_status_str = format!("int_st {}", feature_interrupt_status);

    hal.display().borrow_mut().text(&interrupt_status_str, Point::new(80, 150));

    accel_ex.soft_reset(&mut delay).unwrap();
    accel_ex.init(&mut delay).expect("unable to init bma423");

    let internal_status = accel_ex.read_internal_status().unwrap();
    println!("internal_status = {}", internal_status);

    accel.set_accel_config(
        bma423::AccelConfigOdr::Odr100,
        bma423::AccelConfigBandwidth::NormAvg4,
        bma423::AccelConfigPerfMode::CicAvg,
        bma423::AccelRange::Range2g,
    ).unwrap();

    let axes_config = AxesConfig {
        x_axis: 0,
        x_axis_inv: 0,
        y_axis: 1,
        y_axis_inv: 1,
        z_axis: 2,
        z_axis_inv: 1,
    };

    accel_ex.remap_axes(axes_config).unwrap();
    accel_ex.enable_wrist_tilt().unwrap();

    let int1_cfg = accel_ex.configure_int1_io_ctrl(InterruptIOCtlFlags::OutputEn | InterruptIOCtlFlags::Od).unwrap();
    println!("int1_cfg = {}", int1_cfg);

    accel_ex.map_int1_feature_interrupt(FeatureInterruptStatus::WristWear/* | FeatureInterruptStatus::AnyMotion*/, true).unwrap();

    let feature_config = accel_ex.get_feature_config().unwrap();
    println!("feature_config = {:02X?}", feature_config);
    */

    /* TOUCHPAD
    let proxy_touch = HAL::get_proxy(hal.i2c_man().clone());

    let touch_rst = unsafe { Gpio33::new() };
    let touch_int = unsafe { Gpio12::new() };
    let mut touch = setupTouchpad(touch_rst.downgrade(), touch_int.downgrade(), proxy_touch);

    //let info = touch.get_device_info().unwrap();
    //println!("touch device version = {} info = {:02X?}", info.Version, info.VersionInfo);

    //let (ax, ay, az) = accel.get_x_y_z().unwrap();
    //println!("ax = {} ay = {} az = {}", ax, ay, az);

    //let gestAddr = touch.set_gesture_output_address(0x01).unwrap();
    //println!("touch gest addr = {}", gestAddr);

    let mut data: [u8; 10] = [0; 10];

    for i in 0..100 {
        //touch.get_data_raw(&mut data).unwrap();
        //println!("touch raw data = {:02X?}", data);

        let touch_event = touch.read_one_touch_event(false).unwrap();
        let TouchEvent {x,y,..} = touch_event;

        println!("touch gesture = {:?} x = {} y = {}", touch_event.gesture, x, y);

        let circle_style = PrimitiveStyle::with_fill(Rgb565::RED);
        hal.display().borrow_mut().circle(Point::new(x, y), 5, circle_style);

        thread::sleep(Duration::from_millis(20));
    }
    TOUCHPAD */

    /* RTC
    let proxy_rtc = HAL::get_proxy(hal.i2c_man().clone());

    //let proxy_rtc = RefCellDevice::new(&i2c_ref_cell);
    let mut rtc = PCF8563::new(proxy_rtc.reverse());

    let datetime_rtc = DateTime {
        year: 23,
        month: 1,
        weekday: 1,
        day: 1,
        hours: 0,
        minutes: 0,
        seconds: 0,
    };

    rtc.get_datetime().unwrap();

    let offset = UtcOffset::from_hms(2, 0, 0).unwrap();

    let datetime = Date::from_calendar_date(
        datetime_rtc.year as i32 + 2000,
        Month::try_from(datetime_rtc.month).unwrap(),
        datetime_rtc.day,
    )
    .unwrap()
    .with_hms(
        datetime_rtc.hours,
        datetime_rtc.minutes,
        datetime_rtc.seconds,
    )
    .unwrap()
    .assume_offset(offset);

    print!("rtc: {}", datetime);

    hal.display().borrow_mut().clear();

    let template = format_description!(
        version = 2,
        "[hour repr:24]:[minute]:[second]"
    );

    let text = datetime.format(&template).unwrap();
    let style_time = MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK);

    hal.display().borrow_mut().text_aligned(&text, Point::new(120, 120), style_time, embedded_graphics::text::Alignment::Center);
    RTC */
    let sysloop = EspSystemEventLoop::take().unwrap();

    let nvs_partition = nvs::EspDefaultNvsPartition::take().unwrap();
    //let modem = peripherals.modem;
    //let mut _wifi = setupWifi(sysloop, modem, nvs_partition.clone());

    setup_bluetooth();

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

fn setup_bluetooth() {
    let ble_device = BLEDevice::take();

    let server = ble_device.get_server();
    server.on_connect(|_| {
        ::log::info!("Client connected");
        ::log::info!("Multi-connect support: start advertising");
        ble_device.get_advertising().start().unwrap();
    });
    let service = server.create_service(uuid128!("5e98f6d5-0837-4147-856f-61873c82da9b"));

    // A static characteristic.
    let static_characteristic = service.lock().create_characteristic(
        uuid128!("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93"),
        NimbleProperties::READ,
    );
    static_characteristic
        .lock()
        .set_value("Hello, world!".as_bytes());

    // A characteristic that notifies every second.
    let notifying_characteristic = service.lock().create_characteristic(
        uuid128!("594429ca-5370-4416-a172-d576986defb3"),
        NimbleProperties::READ | NimbleProperties::NOTIFY,
    );
    notifying_characteristic.lock().set_value(b"Initial value.");

    // A writable characteristic.
    let writable_characteristic = service
        .lock()
        .create_characteristic(
        uuid128!("3c9a3f00-8ed3-4bdf-8a39-a01bebede295"),
        NimbleProperties::READ | NimbleProperties::WRITE);

    writable_characteristic
        .lock()
        .on_read(move |val, _| {
            val.set_value("Sample value".as_ref());
            ::log::info!("Read from writable characteristic.");
        })
        .on_write(move |value, _param| {
            handle_incoming(value);
        });

    let ble_advertising = ble_device.get_advertising();
    ble_advertising
        .name("ESP32-SmartWatchTest-123456")
        .add_service_uuid(uuid128!("8b3c29a1-7817-44c5-b001-856a40aba114"));

    ble_advertising.start().unwrap();

    for i in 0..60 {
        notifying_characteristic.lock().set_value(format!("tick {}", i).as_bytes()).notify();
        thread::sleep(Duration::from_millis(1000));
    }
}

fn handle_incoming(buf: &[u8]) {
    let ride : Ride = rmp_serde::from_slice(buf).unwrap();
    println!("Wrote to writable characteristic: {:?}", ride);

    /*
    unsafe
    {
        CurrentCoords = coords;
    }
    */
}

fn setupTouchpad<'d, 'a>(reset_pin: AnyIOPin, int_pin: AnyIOPin, i2c: RefCellDevice<'a, I2cDriver<'d>>) -> CST816S<Reverse<RefCellDevice<'a, I2cDriver<'d>>>, PinDriver<'d, AnyIOPin, Input>, PinDriver<'d, AnyIOPin, Output>> {
    let rst = PinDriver::output(reset_pin).unwrap();
    let int = PinDriver::input(int_pin).unwrap();

    let mut touchpad = CST816S::new(i2c.reverse(), int, rst);

    let mut delay = Ets;

    touchpad.setup(&mut delay).unwrap();

    touchpad
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
