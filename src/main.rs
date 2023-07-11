use std::{error::Error, ptr, thread, time::Duration};
use std::mem::size_of;
use std::time::SystemTime;

//use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_8X13},
        MonoTextStyle,
    },
    pixelcolor::Rgb565,
    prelude::{DrawTarget, RgbColor, *},
    text::Text,
};
use embedded_graphics::mono_font::iso_8859_16::FONT_10X20;
use embedded_svc::storage::RawStorage;
use esp_idf_hal::{delay::Ets, gpio::PinDriver, gpio::{AnyIOPin, Gpio13, Gpio14, Gpio15, Gpio19, Gpio27}, i2c::I2cConfig, i2c::I2cDriver, peripherals::Peripherals, prelude::*, spi::{Dma, SpiDeviceDriver, SpiDriver, SPI2}, delay};

use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use esp_idf_hal::modem::Modem;

use esp_idf_svc::{eventloop::EspSystemEventLoop, log, netif::{EspNetif, EspNetifWait}, nvs, nvs::EspDefaultNvsPartition, ping, sntp, wifi::{EspWifi, WifiWait}};
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
use time::macros::{datetime, offset, format_description};
//use esp_idf_svc::log;
use bma423::{Bma423, FeatureInterruptStatus, Features, InterruptLine, PowerControlFlag};
use embedded_hal::blocking::i2c::{Write, WriteRead};
use embedded_hal::prelude::_embedded_hal_blocking_i2c_Write;

use esp_idf_hal::{
    gpio::{self, Output},
    i2c
};
use esp_idf_svc::log::EspLogger;

mod peripherals {
    pub mod gc9a01;
    pub mod bma423ex;
}

use crate::peripherals::gc9a01::Builder_GC9A01Rgb565;
use crate::peripherals::bma423ex::{AxesConfig, Bma423Ex, InterruptIOCtlFlags};

const SSID: &str = "HOTBOX-B212";
const PASS: &str = "0534337688";
const LAST_SYNC: &str = "last_sync";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LastSyncInfo {
    pub last_sync: OffsetDateTime,
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    EspLogger::initialize_default();

    let wakeup_cause = get_wakeup_cause();
    let wakeup_cause_str = format_wakeup_cause(wakeup_cause);

    let peripherals = Peripherals::take().unwrap();

    let mut delay = Ets;
    let spi = unsafe { SPI2::new() };
    let cs = unsafe { Gpio15::new() };
    let sclk = unsafe { Gpio14::new() };
    let sdo = unsafe { Gpio13::new() };
    let rst = PinDriver::input_output_od(unsafe { Gpio27::new() }).unwrap();
    let dc = PinDriver::input_output_od(unsafe { Gpio19::new() }).unwrap();

    let driver = SpiDriver::new(spi, sclk, sdo, None::<AnyIOPin>, Dma::Disabled).unwrap();

    let spi_config = esp_idf_hal::spi::config::Config::default()
        .baudrate(20_000_000.Hz())
        .write_only(true);

    let spi = SpiDeviceDriver::new(driver, Some(cs), &spi_config).unwrap();

    // create a DisplayInterface from SPI and DC pin, with no manual CS control
    let di = SPIInterfaceNoCS::new(spi, dc);
    // create the ILI9486 display driver in rgb666 color mode from the display interface and use a HW reset pin during init
    let mut display = Builder_GC9A01Rgb565::create(di)
        .init(&mut delay, Some(rst))
        .map_err(|_| Box::<dyn Error>::from("display init"))
        .unwrap();

    let mut backlight = PinDriver::output(peripherals.pins.gpio21).unwrap();
    backlight.set_high().unwrap();

    // clear the display to black
    display
        .clear(Rgb565::BLACK)
        .map_err(|_| Box::<dyn Error>::from("clear display"))
        .unwrap();

    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    // Create a text at position (20, 30) and draw it using the previously defined style
    Text::new("Hello, world!", Point::new(80, 120), style)
        .draw(&mut display)
        .unwrap();

    Text::new(wakeup_cause_str, Point::new(80, 130), style)
        .draw(&mut display)
        .unwrap();

    let scl = peripherals.pins.gpio25;
    let sda = peripherals.pins.gpio26;
    let config = I2cConfig::new().baudrate(100.kHz().into());

    let i2c_driver = I2cDriver::new(peripherals.i2c0, sda, scl, &config).unwrap();

    let i2c_bus: &'static _ = shared_bus::new_std!(I2cDriver = i2c_driver).unwrap();

    let i2c_proxy2 = i2c_bus.acquire_i2c();
    let mut accel = Bma423::new_with_address(i2c_proxy2, 0x18);
    let i2c_proxy3 = i2c_bus.acquire_i2c();
    let mut accel_ex = Bma423Ex::new(i2c_proxy3);

    let interrupt_status = accel.read_interrupt_status().unwrap();
    let feature_interrupt_status: u8 = interrupt_status.feature.into();
    let interrupt_status_str = format!("int_st {}", feature_interrupt_status);

    Text::new(&interrupt_status_str, Point::new(80, 140), style)
        .draw(&mut display)
        .unwrap();

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

    for i in 0..1 {
        let (ax, ay, az) = accel.get_x_y_z().unwrap();

        println!("ax = {} ay = {} az = {}", ax, ay, az);

        thread::sleep(Duration::from_millis(100));
    }

    //let i2c_proxy = i2c_bus.acquire_i2c();
    //let mut rtc = PCF8563::new(i2c_proxy);

    let datetime_rtc = DateTime {
        year: 23,
        month: 1,
        weekday: 1,
        day: 1,
        hours: 0,
        minutes: 0,
        seconds: 0,
    }; //rtc.get_datetime().unwrap();

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

    display
        .clear(Rgb565::BLACK)
        .map_err(|_| Box::<dyn Error>::from("clear display"))
        .unwrap();

    let template = format_description!(
        version = 2,
        "[hour repr:24]:[minute]:[second]"
    );

    let text = datetime.format(&template).unwrap();
    let style_time = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);

    Text::with_alignment(
        &text,
        Point::new(120, 120),
        style_time,
        embedded_graphics::text::Alignment::Center,
    )
    .draw(&mut display)
    .unwrap();

    let sysloop = EspSystemEventLoop::take().unwrap();

    let nvs_partition = nvs::EspDefaultNvsPartition::take().unwrap();
    let mut _wifi = setupWifi(sysloop, peripherals.modem, nvs_partition.clone());

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

        let diff = datetime - last_sync_info.last_sync;

        if diff.whole_days() > 1 {
            //sync_rtc(&mut nvs, &mut rtc);
        }
    }
    else {
        println!("first sync");
        //sync_rtc(&mut nvs, &mut rtc);
    }

    _wifi.disconnect().unwrap();

    unsafe {
        let result = esp_idf_sys::esp_sleep_enable_ext0_wakeup(gpio_num_t_GPIO_NUM_34, 0);
        println!("result {}", result);

        let result_ext1 = esp_idf_sys::esp_sleep_enable_ext1_wakeup(
            1 << 32,
            esp_sleep_ext1_wakeup_mode_t_ESP_EXT1_WAKEUP_ALL_LOW,
        );
        println!("result {}", result_ext1);

        thread::sleep(Duration::from_millis(1500));
        display.clear(Rgb565::BLACK).unwrap();

        println!("going to deep sleep");
        esp_idf_sys::esp_deep_sleep_disable_rom_logging();
        esp_idf_sys::esp_deep_sleep_start();
    };
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
        thread::sleep(Duration::from_millis(1000));
    }

    println!("ip acquired");

    let ip_info = wifi_driver.sta_netif().get_ip_info().unwrap();
    println!("ip: {}", ip_info.ip);
    println!("dns: {:?}", ip_info.dns);

    return wifi_driver;
}

fn sync_rtc<T, E>(nvs: &mut EspNvs<NvsDefault>, rtc: &mut PCF8563<T>)
    where T: Write<Error = E> + WriteRead<Error = E>, E: std::fmt::Debug
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