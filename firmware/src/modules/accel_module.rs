use crate::peripherals::accelerometer::Accelerometer;
use crate::peripherals::i2c_proxy_async::I2cProxyAsync;
use esp_idf_hal::i2c::I2cDriver;
use log::{error, info};
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Notify;

use tokio::time::{sleep, Duration};

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

        let mut accel_init_res = Accelerometer::create(proxy, proxy_ex).await;

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
        accel: Accelerometer<'static>,
        mut commands: Receiver<Commands>,
        events: Sender<Events>,
    ) {
        let mut recv_event = events.subscribe();

        let start_read = Arc::new(Notify::new());

        let mut thermometer = accel.get_thermometer();

        let accel_job = tokio::spawn(Self::read_interrupt_status(accel, start_read.clone()));

        loop {
            tokio::select! {
                Ok(command) = commands.recv() => {
                    info!("{:?}", command);
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
                    info!("{:?}", event);
                    match event {
                        Events::TouchOrMove => {
                            start_read.notify_one();
                        }
                        Events::Wakeup(cause) => {
                        }
                        _ => {}
                    }
                },
            }
        }

        accel_job.abort();
    }

    async fn read_interrupt_status<'a>(accel: Accelerometer<'a>, start_reading: Arc<Notify>) {
        let mut accel_mut = accel;
        loop {
            start_reading.notified().await;
            info!("reading interrupt...");
            while accel_mut.read_interrupt_status() != 0 {
                sleep(Duration::from_millis(100)).await;
            }
            info!("no interrupt");
        }
    }
}
