use std::{
    collections::hash_map::{self},
    hash::{BuildHasher, Hash, Hasher},
    sync::Arc,
};
use tokio::sync::broadcast::Sender;

use crate::peripherals::nvs_storage::NvsStorage;

use log::{error, info};

use blinky_shared::events::Events;
use blinky_shared::{
    commands::Commands,
    error::Error,
    persistence::{PersistenceUnit, PersistenceUnitDto},
};

pub struct PersisterModule {}

const NVS_NAMESPACE: &str = "blinky_persistence";

impl PersisterModule {
    pub async fn start(commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let mut storage = NvsStorage::create(NVS_NAMESPACE);

        info!("start");

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    match command {
                        Commands::Persist(persistence_unit) => {
                            let dto: PersistenceUnitDto = persistence_unit.into();

                            let kind = dto.kind;
                            info!("persisting {:?}", kind);

                            let read_result = storage.read_bytes(kind.as_ref());

                            if let Ok(bytes) = read_result {
                                let existing = PersistenceUnitDto {
                                    kind: kind,
                                    data: bytes
                                };

                                let hasher_state = hash_map::RandomState::new();
                                let mut hasher_of_existing = hasher_state.build_hasher();
                                let mut hasher_of_new = hasher_state.build_hasher();

                                existing.hash(&mut hasher_of_existing);
                                dto.hash(&mut hasher_of_new);

                                if hasher_of_existing.finish() == hasher_of_new.finish() {
                                    info!("persisting of {:?} skipped, same hash", kind);

                                    continue;
                                }
                            }

                            let result = storage
                                .write_bytes::<Vec<u8>>(kind.as_ref(), &dto.data)
                                .map_err(|x| Error::from(x));

                            if let Err(error) = result {
                                error!("{:?}", error);
                            }

                        }
                        Commands::Restore(kind) => {
                            info!("restoring {:?}", kind);

                            let result = storage
                                .read_bytes(kind.as_ref());

                            let persistence_unit = match result {
                                Ok(bytes) => {
                                    PersistenceUnit {
                                        kind: kind,
                                        data: Ok(Arc::new(bytes))
                                    }
                                },
                                Err(err) => {
                                    PersistenceUnit {
                                        kind: kind,
                                        data: Err(Error::from(err))
                                    }
                                }
                            };

                            events.send(Events::Restored(persistence_unit)).unwrap();
                        }
                        Commands::StartDeepSleep => {
                            break;
                        }
                        _ => {}
                    }
                },
                Ok(event) = recv_event.recv() => {
                    match event {
                        _ => {}
                    }
                }
            }
        }

        info!("done.");
    }
}