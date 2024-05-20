use blinky_shared::calendar::CalendarEventKey;
use blinky_shared::contract::packets::{
    CalendarEventSyncResponsePacket, ReferenceDataPacket, ReferenceDataPacketType,
};
use esp32_nimble::utilities::mutex::Mutex;
use esp32_nimble::utilities::BleUuid;
use esp32_nimble::{uuid128, BLEAdvertisementData, BLECharacteristic, BLEDevice, NimbleProperties};
use log::{error, info};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;

use blinky_shared::commands::Commands;
use blinky_shared::events::Events;
use blinky_shared::message_bus::{BusHandler, BusSender, MessageBus};

pub struct BleModule {}

struct Context {
    tx: Sender<BleCommands>,
}

struct BleContext {
    is_ble_initialized: bool,
    rw_characteristic: Option<Arc<Mutex<BLECharacteristic>>>,
}

#[derive(Clone, Debug)]
pub enum BleCommands {
    StartAdvertising,
    Shutdown,
    ReplyPersisted(Arc<Vec<CalendarEventKey>>),
}

impl BusHandler<Context> for BleModule {
    async fn event_handler(_bus: &BusSender, context: &mut Context, event: Events) {
        match event {
            Events::PersistedCalendarEvents(events) => {
                context
                    .tx
                    .send(BleCommands::ReplyPersisted(events))
                    .unwrap();
            }
            _ => {}
        }
    }

    async fn command_handler(_bus: &BusSender, context: &mut Context, command: Commands) {
        match command {
            Commands::RequestReferenceData => {
                context.tx.send(BleCommands::StartAdvertising).unwrap();
            }
            Commands::ShutdownBle | Commands::StartDeepSleep => {
                context.tx.send(BleCommands::Shutdown).unwrap();
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

        let (tx, rx) = channel::<BleCommands>();

        let context = Context { tx };

        let bus_clone = bus.clone();
        let ble_task = tokio::task::spawn_blocking(move || {
            Self::ble_loop(bus_clone, rx);
        });

        MessageBus::handle::<Context, Self>(bus, context).await;

        ble_task.await.unwrap();

        info!("done.");
    }

    fn ble_loop(bus: MessageBus, rx: std::sync::mpsc::Receiver<BleCommands>) {
        let mut context = BleContext {
            rw_characteristic: None,
            is_ble_initialized: false,
        };

        loop {
            match rx.recv() {
                Ok(command) => {
                    Self::handle_ble_command(&bus, &mut context, command);
                }
                Err(err) => {
                    error!("ble error: {:?}", err);
                    return;
                }
            }
        }
    }

    fn handle_ble_command(bus: &MessageBus, context: &mut BleContext, command: BleCommands) {
        match command {
            BleCommands::StartAdvertising => {
                let rw = Self::start_ble_advertising(bus);

                if let Some(ch) = rw {
                    let _ = context.rw_characteristic.insert(ch);

                    context.is_ble_initialized = true;
                }
            }
            BleCommands::Shutdown => {
                if context.is_ble_initialized {
                    Self::shutdown_ble();
                    context.is_ble_initialized = false;
                }
            }
            BleCommands::ReplyPersisted(events) => {
                if context.is_ble_initialized {
                    Self::reply_persisted(context, events);
                    Self::shutdown_ble();
                    context.is_ble_initialized = false;
                } else {
                    error!("reply_persisted skipped!");
                }
            }
        }
    }

    fn start_ble_advertising(bus: &MessageBus) -> Option<Arc<Mutex<BLECharacteristic>>> {
        info!("initializing bluetooth...");

        let ble_device = BLEDevice::take();

        let server = ble_device.get_server();

        server.ble_gatts_show_local();
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
            NimbleProperties::READ | NimbleProperties::WRITE | NimbleProperties::NOTIFY,
        );

        let bus = bus.clone();
        rw_characteristic.lock().on_write(move |args| {
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
            return None;
        }

        info!("advertising...");

        advertising
            .lock()
            .on_complete(|x| info!("advertising completed."));

        return Some(rw_characteristic);
    }

    fn shutdown_ble() {
        info!("shutting down BLE...");

        let ble_device = BLEDevice::take();
        let server = ble_device.get_server();

        let connections = server.connections();

        for connection in connections {
            info!("client {:?}", connection.address());
        }

        if let Err(err) = BLEDevice::deinit() {
            error!("{:?}", err);
        }

        info!("BLE shut down.");
    }

    fn reply_persisted(context: &BleContext, events: Arc<Vec<CalendarEventKey>>) {
        info!("replying persisted {} events...", events.len());

        let packets = events.iter().map(|x| {
            let packet = CalendarEventSyncResponsePacket {
                kind: x.0,
                event_id: x.1,
            };

            let reference_packet = ReferenceDataPacket::wrap(
                ReferenceDataPacketType::CalendarEventsSyncResponse,
                packet,
            );

            let buf = reference_packet.serialize();

            buf
        });

        for packet in packets {
            let buf = packet.as_slice();

            info!("replying packet: {:02X?}", &buf);

            context
                .rw_characteristic
                .as_ref()
                .unwrap()
                .lock()
                .set_value(buf);

            context.rw_characteristic.as_ref().unwrap().lock().notify();
        }

        info!("replying persisted complete.");
    }
}
