#![feature(slice_as_chunks)]
#![feature(vec_push_within_capacity)]
#![feature(associated_type_bounds)]
#![feature(type_alias_impl_trait)]
#![feature(associated_type_defaults)]
#![feature(generic_arg_infer)]

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;
use blinky_shared::fasttrack::FastTrackRtcData;
use blinky_shared::message_bus::MessageBus;
use blinky_shared::modules::calendar_module::CalendarModule;
use blinky_shared::modules::reference_time::ReferenceTime;
use blinky_shared::modules::renderer::Renderer;
use blinky_shared::persistence::PersistenceUnitKind;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::log::{set_target_level, EspLogger};
use log::*;
use modules::rtc_display_fasttrack::RtcDisplayFastTrack;
use peripherals::pins::mapping::PinsMapping;
use peripherals::pins::twatch_2021::TWatch2021Pins;
use peripherals::rtc::Rtc;
use std::future::Future;
use std::pin::Pin;
use std::thread;

extern crate blinky_shared;

mod modules;
mod peripherals;

use peripherals::hal::{HalConfig, PinConfig, HAL};

use modules::accel_module::AccelerometerModule;
use modules::ble_module::BleModule;
use modules::persister_module::PersisterModule;
use modules::power_module::PowerModule;
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
        //.worker_threads(2)
        .build()?;

    rt.block_on(main_async())?;

    PowerModule::goto_deep_sleep();

    Ok(())
}

async fn main_async() -> Result<(), Box<dyn std::error::Error>> {
    info!("main_async...");

    let peripherals = Peripherals::take().unwrap();

    let mut pins_mapping = TWatch2021Pins::new(peripherals.pins);

    let hal_conf = HalConfig {
        touch_interrupt_pin: 12,
        touch_reset_pin: 33,
    };

    let pin_conf = PinConfig { vibro: 4 };

    let message_bus = MessageBus::new();

    let hal: HAL = HAL::new(hal_conf.clone(), peripherals.i2c0);

    let logging_task = start_logging(&message_bus);

    let mut mb = message_bus.clone();
    let wait_for_first_render_task = mb.wait_for(Events::FirstRender);

    let i2c_proxy = hal.get_i2c_proxy_async().clone();
    let fasttrack_result = RtcDisplayFastTrack::run_and_decompose(i2c_proxy, &mut pins_mapping);

    let rtc_task = start_rtc(&message_bus, fasttrack_result.rtc);

    let renderer_task = start_renderer(
        &message_bus,
        fasttrack_result.display,
        fasttrack_result.rtc_data,
    );

    let tasks_batch: Vec<Pin<Box<dyn futures::Future<Output = ()>>>> = vec![
        Box::pin(logging_task),
        Box::pin(wait_for_first_render_task),
        Box::pin(rtc_task),
        Box::pin(renderer_task),
    ];

    let (_, _, mut remaining_tasks) = futures::future::select_all(tasks_batch).await;

    let time_sync_task = start_time_sync(&message_bus);

    let mb = message_bus.clone();
    let persister_task = PersisterModule::start(mb);

    let mb = message_bus.clone();
    let ble_task = BleModule::start(mb);

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

    let startup_sequence = async move {
        mb.send_cmd(Commands::Restore(PersistenceUnitKind::RtcSyncInfo));
        mb.send_cmd(Commands::SyncCalendar);
    };

    let mb = message_bus.clone();

    let power_task = PowerModule::start(peripherals.adc1, &mut pins_mapping, pin_conf, mb);

    let mut rest: Vec<Pin<Box<dyn futures::Future<Output = ()>>>> = vec![
        Box::pin(power_task),
        Box::pin(time_sync_task),
        Box::pin(persister_task),
        Box::pin(accel_task),
        Box::pin(ble_task),
        Box::pin(user_input_task),
        Box::pin(touch_task),
        Box::pin(reference_time_task),
        Box::pin(calendar_task),
        Box::pin(startup_sequence),
    ];

    remaining_tasks.append(&mut rest);

    futures::future::join_all(remaining_tasks).await;

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

fn start_rtc(mb: &MessageBus, rtc: Rtc<'static>) -> impl Future<Output = ()> {
    let mb = mb.clone();
    RtcModule::start(rtc, mb)
}

fn start_renderer(
    mb: &MessageBus,
    display: ClockDisplay<'static>,
    rtc_data: FastTrackRtcData,
) -> impl Future<Output = ()> {
    let mb = mb.clone();
    Renderer::<ClockDisplay>::start(mb, display, rtc_data)
}
