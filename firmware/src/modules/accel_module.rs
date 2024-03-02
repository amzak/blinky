use crate::peripherals::accelerometer::{Accelerometer, Thermometer};
use crate::peripherals::i2c_proxy_async::I2cProxyAsync;
use blinky_shared::message_bus::{BusHandler, BusSender, MessageBus};
use esp_idf_hal::i2c::I2cDriver;
use log::{error, info};

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;

pub struct AccelerometerModule {}

struct Context<'a> {
    accel: Accelerometer<'a>,
    thermometer: Thermometer<'a>,
}

impl<'a> BusHandler<Context<'a>> for AccelerometerModule {
    async fn event_handler(_: &BusSender, context: &mut Context<'a>, event: Events) {
        match event {
            Events::TouchOrMove => {
                Self::read_interrupt_status(&mut context.accel).await;
            }
            _ => {}
        }
    }

    async fn command_handler(bus: &BusSender, context: &mut Context<'a>, command: Commands) {
        match command {
            Commands::GetTemperature => {
                let temperature = context.thermometer.read_temperature();
                bus.send_event(Events::Temperature(temperature));
            }
            _ => {}
        }
    }
}

impl AccelerometerModule {
    pub async fn start(
        proxy: I2cProxyAsync<I2cDriver<'static>>,
        proxy_ex: I2cProxyAsync<I2cDriver<'static>>,
        bus: MessageBus,
    ) {
        info!("starting...");

        let accel_init_res = tokio::spawn(Accelerometer::create(proxy, proxy_ex))
            .await
            .unwrap();

        match accel_init_res {
            Ok(accel) => {
                let thermometer = accel.get_thermometer();
                let context = Context { accel, thermometer };

                MessageBus::handle::<Context, Self>(bus, context).await;
            }
            Err(err) => {
                error!("{}", err);
            }
        }

        info!("done.")
    }

    async fn read_interrupt_status<'a>(accel: &mut Accelerometer<'a>) {
        let interrupt_status = accel.read_interrupt_status();
        info!("accel interrupt status = {:?}", interrupt_status);
    }
}
