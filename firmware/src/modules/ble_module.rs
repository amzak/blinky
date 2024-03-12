use esp32_nimble::utilities::BleUuid;
use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties};
use log::{error, info, warn};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;
use blinky_shared::message_bus::{BusHandler, BusSender, MessageBus};

pub struct BleModule {}

struct Context {
    tx: Sender<Commands>,
}

impl BusHandler<Context> for BleModule {
    async fn event_handler(_bus: &BusSender, _context: &mut Context, _event: Events) {}

    async fn command_handler(_bus: &BusSender, context: &mut Context, command: Commands) {
        match command {
            Commands::RequestReferenceData | Commands::ShutdownBle | Commands::StartDeepSleep => {
                if let Err(err) = context.tx.send(command) {
                    error!("{:?}", err);
                }
            }
            _ => {}
        }
    }
}

impl BleModule {
    const DEVICE_NAME: &'static str = "ESP32-SmartWatchTest-123456";

    const SERVICE_GUID: BleUuid = uuid128!("5e98f6d5-0837-4147-856f-61873c82da9b");

    const STATIC_CHARACTERISTIC: BleUuid = uuid128!("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93");
    const NOTIFYING_CHARACTERISTIC: BleUuid = uuid128!("594429ca-5370-4416-a172-d576986defb3");
    const RW_CHARACTERISTIC: BleUuid = uuid128!("3c9a3f00-8ed3-4bdf-8a39-a01bebede295");

    pub async fn start(bus: MessageBus) {
        info!("starting...");

        let (tx, rx) = channel::<Commands>();

        let context = Context { tx };

        let bus_clone = bus.clone();
        let ble_task = tokio::task::spawn_blocking(move || {
            Self::setup_bluetooth(bus_clone, rx);
        });

        MessageBus::handle::<Context, Self>(bus, context).await;

        ble_task.await.unwrap();

        info!("done.");
    }

    fn setup_bluetooth(bus: MessageBus, rx: std::sync::mpsc::Receiver<Commands>) {
        let command = rx.recv().unwrap();

        if matches!(command, Commands::StartDeepSleep) {
            return;
        }

        info!("initializing bluetooth...");

        let ble_device = BLEDevice::take();

        let server = ble_device.get_server();
        server.advertise_on_disconnect(false);

        let bus_clone = bus.clone();
        server.on_connect(move |_server, _desc| {
            info!("client connected");
            bus_clone.send_event(Events::BleClientConnected);
        });

        let bus_clone = bus.clone();
        server.on_disconnect(move |_server, _desc| {
            info!("client disconnected");
            bus_clone.send_event(Events::BleClientDisconnected);
        });

        let service = server.create_service(Self::SERVICE_GUID);

        let static_characteristic = service
            .lock()
            .create_characteristic(Self::STATIC_CHARACTERISTIC, NimbleProperties::READ);
        static_characteristic
            .lock()
            .set_value("Hello, world!".as_bytes());

        let notifying_characteristic = service.lock().create_characteristic(
            Self::NOTIFYING_CHARACTERISTIC,
            NimbleProperties::READ | NimbleProperties::NOTIFY,
        );
        notifying_characteristic.lock().set_value(b"Initial value.");

        let rw_characteristic = service.lock().create_characteristic(
            Self::RW_CHARACTERISTIC,
            NimbleProperties::READ | NimbleProperties::WRITE,
        );

        rw_characteristic
            .lock()
            .on_read(move |val, _| {
                val.set_value("Sample value".as_ref());
                info!("Read from writable characteristic.");
            })
            .on_write(move |args| {
                let data = args.recv_data();
                bus.send_event(Events::IncomingData(Arc::new(Vec::from(data))));
            });

        let advertising = ble_device.get_advertising();

        let mut ad_data = BLEAdvertisementData::new();
        ad_data
            .name(Self::DEVICE_NAME)
            .add_service_uuid(Self::SERVICE_GUID);

        advertising.lock().set_data(&mut ad_data).unwrap();

        if let Err(error) = advertising.lock().start() {
            error!("can't start ble advertising, error {:?}", error);
            return;
        }

        info!("advertising...");

        advertising
            .lock()
            .on_complete(|x| info!("advertising completed."));

        /*
        for i in 0..60 {
            notifying_characteristic.lock().set_value(format!("tick {}", i).as_bytes()).notify();
            sleep(Duration::from_millis(1000)).await;
        }
        */

        let command = rx.recv().unwrap();

        if let Err(err) = BLEDevice::deinit_full() {
            error!("{:?}", err);
        }
    }
}
