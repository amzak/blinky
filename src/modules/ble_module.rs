use esp32_nimble::{BLEDevice, NimbleProperties, uuid128};
use esp32_nimble::utilities::BleUuid;
use log::info;
use tokio::sync::broadcast::Sender;
use crate::peripherals::hal::{Commands, Events};

pub struct BleModule {

}

impl BleModule {
    const DEVICE_NAME: &str = "ESP32-SmartWatchTest-123456";

    const SERVICE_GUID: BleUuid = uuid128!("5e98f6d5-0837-4147-856f-61873c82da9b");
    const AD_SERVICE_GUID: BleUuid = uuid128!("8b3c29a1-7817-44c5-b001-856a40aba114");

    const STATIC_CHARACTERISTIC: BleUuid = uuid128!("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93");
    const NOTIFYING_CHARACTERISTIC: BleUuid = uuid128!("594429ca-5370-4416-a172-d576986defb3");
    const RW_CHARACTERISTIC: BleUuid = uuid128!("3c9a3f00-8ed3-4bdf-8a39-a01bebede295");

    pub async fn start(commands_channel: Sender<Commands>, events_channel: Sender<Events>) {
        let mut recv_cmd = commands_channel.subscribe();
        let mut recv_events = events_channel.subscribe();

        let mut ble_initialized = false;

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    match command {
                        Commands::RequestReferenceData => {
                            info!("{:?} {:?}", command, ble_initialized);
                            if !ble_initialized {
                                Self::setup_bluetooth(events_channel.clone()).await;
                                ble_initialized = true;
                            }
                        }
                        Commands::StartDeepSleep => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        if ble_initialized {
            BLEDevice::deinit();
        }

        info!("done.");
    }

    async fn setup_bluetooth(events_channel: Sender<Events>) {
        info!("initializing bluetooth...");

        let ble_device = BLEDevice::take();

        let server = ble_device.get_server();

        let events= events_channel.clone();
        server.on_connect(move |server, desc| {
            info!("client connected");
            events.send(Events::BluetoothConnected).unwrap();
        });

        let service = server.create_service(Self::SERVICE_GUID);

        // A static characteristic.
        let static_characteristic = service.lock().create_characteristic(
            Self::STATIC_CHARACTERISTIC,
            NimbleProperties::READ,
        );
        static_characteristic
            .lock()
            .set_value("Hello, world!".as_bytes());

        // A characteristic that notifies every second.
        let notifying_characteristic = service.lock().create_characteristic(
            Self::NOTIFYING_CHARACTERISTIC,
            NimbleProperties::READ | NimbleProperties::NOTIFY,
        );
        notifying_characteristic.lock().set_value(b"Initial value.");

        let rw_characteristic = service
            .lock()
            .create_characteristic(
                Self::RW_CHARACTERISTIC,
                NimbleProperties::READ | NimbleProperties::WRITE);

        rw_characteristic
            .lock()
            .on_read(move |val, _| {
                val.set_value("Sample value".as_ref());
                info!("Read from writable characteristic.");
            })
            .on_write(move |args| {
                events_channel.send(Events::IncomingData(Vec::from(args.recv_data))).unwrap();
            });

        let advertising = ble_device
            .get_advertising();

        if !advertising.is_advertising() {
            advertising
                .name(Self::DEVICE_NAME)
                .add_service_uuid(Self::SERVICE_GUID)
                .add_tx_power()
                .start()
                .unwrap();
        }

        info!("advertising...")

        /*
        for i in 0..60 {
            notifying_characteristic.lock().set_value(format!("tick {}", i).as_bytes()).notify();
            sleep(Duration::from_millis(1000)).await;
        }
        */
    }
}