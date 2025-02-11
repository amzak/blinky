#![feature(array_ptr_get)]
#![feature(iter_array_chunks)]

use std::{
    ffi::{self},
    ptr::addr_of,
};

use esp_idf_hal::prelude::Peripherals;
use peripherals::pins::mapping::PinsMapping;
use peripherals::pins::tdisplay143::TDisplay143;

//mod bma423_tests;
mod display_tests;

#[link_section = ".rtc.data"]
pub static mut COUNTER: i32 = 0;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    unsafe {
        log::info!("deep_sleep counter = {}", COUNTER);
    }

    let mut pins_mapping = TDisplay143::new(peripherals.pins);

    //bma423_tests::run(peripherals.i2c0);
    display_tests::run(peripherals.spi2, &mut pins_mapping);

    log::info!("going to deep sleep...");

    unsafe {
        //esp_idf_sys::esp_set_deep_sleep_wake_stub(Some(custom_deep_sleep_wake_stub));

        //esp_idf_sys::esp_deep_sleep(10_000_000);
    }

    log::info!("OK.");
}

#[link_section = ".rtc.data"]
pub static mut cstring: [ffi::c_char; 6] = [72, 69, 76, 76, 79, 0];

#[link_section = ".rtc.text"]
#[no_mangle]
pub extern "C" fn custom_deep_sleep_wake_stub() {
    //let peripherals = Peripherals::take().unwrap();

    unsafe {
        COUNTER += 1;

        let ptr = addr_of!(cstring);

        //esp_idf_sys::esp_rom_install_channel_putc(1, esp_idf_sys::esp_rom_uart_putc);

        //esp_idf_sys::esp_rom_printf(ptr.as_ptr());

        //esp_idf_sys::esp_rom_delay_us(20_000);

        //esp_idf_sys::esp_wake_deep_sleep();
    }
}
