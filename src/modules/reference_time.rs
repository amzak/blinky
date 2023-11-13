use std::ops::Add;
use time::{OffsetDateTime, UtcOffset};
use tokio::sync::broadcast::Sender;
use crate::peripherals::hal::{Commands, Events};
use log::info;
use tokio::time::Duration;
use serde::{Deserialize};

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct GpsCoordinates {
    pub lat: f32,
    pub lon: f32
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct ReferenceTime {
    pub now: i64,
    pub offset_seconds: i32
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct ReferenceData {
    pub reference_time: ReferenceTime,
    pub coordinates: GpsCoordinates
}

impl ReferenceTime {
    pub async fn start(commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    match command {
                        Commands::GetReferenceTime => {
                            info!("{:?}", command);
                            commands.send(Commands::RequestReferenceData).unwrap();
                        }
                        Commands::StartDeepSleep => {
                            break;
                        }
                        _ => {}
                    }
                },
                Ok(event) = recv_event.recv() => {
                    match event {
                        Events::IncomingData(data) => {
                            //info!("{:?}", &event);

                            let reference_data : ReferenceData = rmp_serde::from_slice(&data).unwrap();

                            info!("{:?}", reference_data);

                            let reference_time = reference_data.reference_time;

                            let offset = Duration::from_secs(reference_time.offset_seconds as u64);

                            let offset_from_utc = UtcOffset::from_whole_seconds(reference_time.offset_seconds).unwrap();

                            let now = OffsetDateTime::from_unix_timestamp(reference_time.now)
                                .unwrap()
                                .add(offset)
                                .replace_offset(offset_from_utc);

                            events.send(Events::ReferenceTime(now)).unwrap();
                        }
                        _ => {}
                    }
                }
            }
        }

        info!("done.");
    }
}