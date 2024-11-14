use crate::peripherals::accelerometer::Accelerometer;
use blinky_shared::message_bus::{BusHandler, BusSender, MessageBus};
use esp_idf_hal::i2c::I2cDriver;
use log::{error, info};

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;
use peripherals::i2c_proxy_async::I2cProxyAsync;

pub struct AccelerometerModule {}

struct Context<'a> {
    accel: Accelerometer<'a>,
}

impl<'a> BusHandler<Context<'a>> for AccelerometerModule {
    async fn event_handler(bus: &BusSender, context: &mut Context<'a>, event: Events) {
        match event {
            Events::SharedInterrupt => {
                let int_status = context.accel.read_interrupt_status();
                info!("int_status: {:?}", int_status);
                bus.send_event(Events::AccelerometerInterrupt(int_status.feature.into()));
            }
            _ => {}
        }
    }

    async fn command_handler(bus: &BusSender, context: &mut Context<'a>, command: Commands) {
        match command {
            Commands::DebugAccel => {
                let accel = context.accel.read_accel();
                info!("accel: {:?}", accel);
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
                info!("accelerometer initialized");

                let context = Context { accel };

                bus.send_event(Events::Temperature(context.accel.temperature));

                MessageBus::handle::<Context, Self>(bus, context).await;
            }
            Err(err) => {
                info!("accelerometer initialization failed");
                error!("{}", err);
            }
        }

        info!("done.")
    }
}
