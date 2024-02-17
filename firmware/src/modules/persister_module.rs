use std::{
    collections::hash_map::{self},
    hash::{BuildHasher, Hash, Hasher},
    sync::Arc,
};

use crate::peripherals::nvs_storage::NvsStorage;

use log::{error, info};

use blinky_shared::message_bus::{BusHandler, MessageBus};
use blinky_shared::{
    commands::Commands,
    error::Error,
    persistence::{PersistenceUnit, PersistenceUnitDto},
};
use blinky_shared::{events::Events, message_bus::BusSender};

pub struct PersisterModule {}

const NVS_NAMESPACE: &str = "blinky_persistence";

pub struct Context {
    storage: NvsStorage,
}

impl BusHandler<Context> for PersisterModule {
    async fn event_handler(_bus: &BusSender, _context: &mut Context, _event: Events) {}

    async fn command_handler(bus: &BusSender, context: &mut Context, command: Commands) {
        match command {
            Commands::Persist(persistence_unit) => {
                let dto: PersistenceUnitDto = persistence_unit.into();

                let kind = dto.kind;
                info!("persisting {:?}", kind);

                let read_result = context.storage.read_bytes(kind.as_ref());

                if let Ok(bytes) = read_result {
                    let existing = PersistenceUnitDto {
                        kind: kind,
                        data: bytes,
                    };

                    let hasher_state = hash_map::RandomState::new();
                    let mut hasher_of_existing = hasher_state.build_hasher();
                    let mut hasher_of_new = hasher_state.build_hasher();

                    existing.hash(&mut hasher_of_existing);
                    dto.hash(&mut hasher_of_new);

                    if hasher_of_existing.finish() == hasher_of_new.finish() {
                        info!("persisting of {:?} skipped, same hash", kind);

                        return;
                    }
                }

                let result = context
                    .storage
                    .write_bytes::<Vec<u8>>(kind.as_ref(), &dto.data)
                    .map_err(|x| Error::from(x));

                if let Err(error) = result {
                    error!("{:?}", error);
                }
            }
            Commands::Restore(kind) => {
                info!("restoring {:?}", kind);

                let result = context.storage.read_bytes(kind.as_ref());

                let persistence_unit = match result {
                    Ok(bytes) => PersistenceUnit {
                        kind: kind,
                        data: Ok(Arc::new(bytes)),
                    },
                    Err(err) => PersistenceUnit {
                        kind: kind,
                        data: Err(Error::from(err)),
                    },
                };

                bus.send_event(Events::Restored(persistence_unit));
            }
            _ => {}
        }
    }
}

impl PersisterModule {
    pub async fn start(mut bus: MessageBus) {
        info!("starting...");

        let storage = NvsStorage::create(NVS_NAMESPACE);

        let context = Context { storage };

        MessageBus::handle::<Context, Self>(bus, context).await;

        info!("done.");
    }
}
