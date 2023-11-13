use std::sync::Arc;
use esp_idf_hal::i2c::I2cDriver;
use log::info;
use tokio::sync::broadcast::Sender;
use tokio::sync::Notify;
use crate::peripherals::accelerometer::Accelerometer;
use crate::peripherals::hal::{Commands, Events};
use crate::peripherals::i2c_proxy_async::I2cProxyAsync;

use tokio::time::{sleep, Duration};

pub struct AccelerometerModule {

}

impl AccelerometerModule {
    pub async fn start(proxy: I2cProxyAsync<I2cDriver<'static>>, proxy_ex: I2cProxyAsync<I2cDriver<'static>>, commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let mut accel = Accelerometer::create(proxy, proxy_ex);

        let temperature = accel.read_temperature();

        events.send(Events::Temperature(temperature)).unwrap();

        let start_read = Arc::new(Notify::new());

        let accel_job = tokio::spawn(Self::read_interrupt_status(accel, start_read.clone()));

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    match command {
                        Commands::StartDeepSleep => {
                            break;
                        }
                        _ => {}
                    }
                },
                Ok(event) = recv_event.recv() => {
                    match event {
                        Events::TouchOrMove => {
                            start_read.notify_one();
                        }
                        _ => {}
                    }
                },
            }
        }

        accel_job.abort();

        info!("done.")
    }
    async fn read_interrupt_status(accel: Accelerometer<'_>, start_reading: Arc<Notify>) {
        let mut accel_mut = accel;
        loop {
            start_reading.notified().await;
            while accel_mut.read_interrupt_status() != 0 {
                sleep(Duration::from_millis(1)).await;
            }
        }
    }
}