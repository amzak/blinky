use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use embedded_svc::event_bus::EventBus;
use esp32_nimble::{BLEDevice, NimbleProperties, uuid128};
use tokio::sync::broadcast::Sender;

use crate::peripherals::hal::Commands;

pub struct Bluetooth {
    state: BluetoothState,
    channel: Sender<Commands>
}

pub struct BluetoothConfig {

}

pub enum BluetoothState {
    Uninitialized,
    Idle,
    Connecting,
    Syncing,
    Completed
}

#[derive(Clone, Debug)]
struct IncomingData {
    data: Vec<u8>
}

#[derive(Clone, Debug)]
pub struct BluetoothConnectRequest {
}

pub enum BluetoothConnectResponse {
    Accepted,
    Rejected
}

#[derive(Clone, Debug)]
pub struct BluetoothConnectEvent {

}

#[derive(Clone, Debug)]
pub struct BluetoothConnectEvent2 {

}

impl Clone for BluetoothConfig {
    fn clone(&self) -> Self {
        BluetoothConfig {

        }
    }
}

impl Copy for BluetoothConfig {

}

impl Bluetooth {
    pub fn create(config: BluetoothConfig, hal_channel: Sender<Commands>) -> Self {
        Self {
            state: BluetoothState::Uninitialized,
            channel: hal_channel
        }
    }

    async fn setup_bluetooth() {
        let ble_device = BLEDevice::take();

        let server = ble_device.get_server();
        server.on_connect(|_, _| {
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
            .on_write(move |args| {
                Self::handle_incoming(args.recv_data);
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

    pub async fn start(&self) {
        let channel = &self.channel;
        let mut recv = channel.subscribe();

        tokio::spawn(async move {
            loop {
                let res= recv.recv().await;
                let command = res.unwrap();

                match command {
                    Commands::RequestBluetoothConnection => {
                        Self::setup_bluetooth().await;
                    }
                    _ => {}
                }
            }
        });
    }

    pub fn request_sync() {

    }

    fn handle_incoming(buf: &[u8]) {
        //let ride : Ride = rmp_serde::from_slice(buf).unwrap();
        //println!("Wrote to writable characteristic: {:?}", ride);

        /*
        unsafe
        {
            CurrentCoords = coords;
        }
        */
    }

}