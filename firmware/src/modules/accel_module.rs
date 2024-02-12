use crate::peripherals::accelerometer::Accelerometer;
use crate::peripherals::i2c_proxy_async::I2cProxyAsync;
use esp_idf_hal::i2c::I2cDriver;
use log::{error, info};
use tokio::sync::broadcast::{Receiver, Sender};

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;

pub struct AccelerometerModule {}

impl AccelerometerModule {
    pub async fn start(
        proxy: I2cProxyAsync<I2cDriver<'static>>,
        proxy_ex: I2cProxyAsync<I2cDriver<'static>>,
        commands: Sender<Commands>,
        events: Sender<Events>,
    ) {
        let recv_cmd = commands.subscribe();

        let accel_init_res = Accelerometer::create(proxy, proxy_ex).await;

        match accel_init_res {
            Ok(accel) => {
                Self::proceed(accel, recv_cmd, events).await;
            }
            Err(err) => {
                error!("{}", err);
                info!("error {}", err);
            }
        }

        info!("done.")
    }
    async fn proceed(
        mut accel: Accelerometer<'static>,
        mut commands: Receiver<Commands>,
        events: Sender<Events>,
    ) {
        let mut recv_event = events.subscribe();

        let mut thermometer = accel.get_thermometer();

        loop {
            tokio::select! {
                Ok(command) = commands.recv() => {
                    match command {
                        Commands::StartDeepSleep => {
                            break;
                        }
                        Commands::GetTemperature => {
                            let temperature = thermometer.read_temperature();
                            events.send(Events::Temperature(temperature)).unwrap();
                        }
                        _ => {}
                    }
                },
                Ok(event) = recv_event.recv() => {
                    match event {
                        Events::TouchOrMove => {
                            Self::read_interrupt_status(&mut accel).await;
                        }
                        Events::Wakeup(_) => {
                        }
                        _ => {}
                    }
                },
            }
        }
    }

    async fn read_interrupt_status<'a>(accel: &mut Accelerometer<'a>) {
        let interrupt_status = accel.read_interrupt_status();
        info!("accel interrupt status = {:?}", interrupt_status);
    }
}
