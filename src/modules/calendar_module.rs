use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
};

use crate::{error::Error, modules::reference_time::ReferenceTimeUtc};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset};
use tokio::sync::broadcast::Sender;

use crate::peripherals::hal::{Commands, Events};

use crate::persistence::{PersistenceUnit, PersistenceUnitKind};

pub struct CalendarModule {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarStateDto {
    pub version: i32,
    pub last_sync: ReferenceTimeUtc,
    pub events: Vec<CalendarEventDto>,
}

impl CalendarStateDto {
    pub fn new(events: Vec<CalendarEventDto>, last_sync: ReferenceTimeUtc) -> Self {
        Self {
            version: 1,
            events,
            last_sync,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CalendarEventDto {
    pub id: i64,
    pub title: String,
    pub start: ReferenceTimeUtc,
    pub end: ReferenceTimeUtc,
}

impl From<CalendarEvent> for CalendarEventDto {
    fn from(value: CalendarEvent) -> Self {
        CalendarEventDto {
            id: value.id,
            title: value.title,
            start: value.start.into(),
            end: value.end.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CalendarEvent {
    pub id: i64,
    pub title: String,
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
}

impl Hash for CalendarEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl CalendarEvent {
    pub fn new(dto: CalendarEventDto, tz: UtcOffset) -> CalendarEvent {
        CalendarEvent {
            id: dto.id,
            start: dto.start.to_offset_dt(tz),
            end: dto.end.to_offset_dt(tz),
            title: dto.title,
        }
    }
}

impl From<CalendarEventDto> for CalendarEvent {
    fn from(dto: CalendarEventDto) -> Self {
        CalendarEvent {
            id: dto.id,
            start: dto.start.into(),
            end: dto.end.into(),
            title: dto.title,
        }
    }
}

impl PartialEq<Self> for CalendarEvent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for CalendarEvent {}

impl CalendarModule {
    pub async fn start(commands: Sender<Commands>, events: Sender<Events>) {
        let mut recv_cmd = commands.subscribe();
        let mut recv_event = events.subscribe();

        let mut calendar_events: HashSet<CalendarEvent> = HashSet::new();

        let mut now: Option<OffsetDateTime> = None;
        let mut utc_offset: Option<UtcOffset> = None;

        let mut expected_reference_events_count: Option<i32> = None;
        let mut actual_reference_events: i32 = 0;

        loop {
            tokio::select! {
                Ok(command) = recv_cmd.recv() => {
                    info!("{:?}", command);
                    match command {
                        Commands::SyncCalendar => {
                            events.send(Events::InSync(false)).unwrap();
                        }
                        Commands::StartDeepSleep => {
                            break;
                        }
                        _ => {}
                    }
                },
                Ok(event) = recv_event.recv() => {
                    info!("{:?}", event);
                    match event {
                        Events::TimeNow(time_now) => {
                            now = Some(time_now);
                        }
                        Events::ReferenceCalendarEventsCount(events_count) => {
                            expected_reference_events_count = Some(events_count);
                        }
                        Events::ReferenceCalendarEvent(reference_calendar_event) => {
                            let replaced = calendar_events.replace(reference_calendar_event.clone());

                            if replaced.is_some() {
                                info!("event updated {}", replaced.unwrap().id);
                            }

                            events.send(Events::CalendarEvent(reference_calendar_event)).unwrap();

                            actual_reference_events += 1;

                            if expected_reference_events_count.is_some() && actual_reference_events == expected_reference_events_count.unwrap() {
                                Self::persist_events(&commands, Vec::from_iter(calendar_events.iter().map(|x| x.clone())), &now);
                            }
                        }
                        Events::Timezone(offset) => {
                            utc_offset = Some(UtcOffset::from_whole_seconds(offset).unwrap());
                            commands
                                .send(Commands::Restore(PersistenceUnitKind::CalendarSyncInfo))
                                .unwrap();
                        }
                        Events::Restored(unit) => {
                            if !matches!(unit.kind, PersistenceUnitKind::CalendarSyncInfo) {
                                continue;
                            }

                            if let Err(error) = unit.data {
                                error!("{}", error);
                                continue;
                            }

                            if utc_offset.is_none() {
                                warn!("utc_offset is not set");
                                continue;
                            }

                            Self::try_restore(&events, &mut calendar_events, unit, utc_offset.unwrap());
                        }
                        _ => {}
                    }
                }
            }
        }

        info!("done.");
    }

    fn try_restore(
        events: &Sender<Events>,
        calendar_events: &mut HashSet<CalendarEvent>,
        posponed_restore: PersistenceUnit,
        utc_offset: UtcOffset,
    ) -> bool {
        let res: Result<CalendarStateDto, Error> = posponed_restore.deserialize();

        match res {
            Ok(calendar_info_restored) => {
                info!("{:?}", calendar_info_restored);

                let calendar_events_restored = calendar_info_restored
                    .events
                    .into_iter()
                    .map(|x| CalendarEvent::new(x, utc_offset));

                for event in calendar_events_restored {
                    calendar_events.insert(event.clone());
                    events.send(Events::CalendarEvent(event)).unwrap();
                }
            }
            Err(error) => {
                error!("{:?}", error);
                return false;
            }
        }

        return true;
    }

    fn persist_events(
        commands: &Sender<Commands>,
        mut calendar_events: Vec<CalendarEvent>,
        now: &Option<OffsetDateTime>,
    ) {
        if now.is_none() {
            return;
        }

        let now = now.unwrap();

        calendar_events.retain(|x| x.end >= now);

        let dtos: Vec<CalendarEventDto> = calendar_events.into_iter().map(|x| x.into()).collect();
        let calendar_state_dto = CalendarStateDto::new(dtos, now.into());
        let persistence_unit =
            PersistenceUnit::new(PersistenceUnitKind::CalendarSyncInfo, &calendar_state_dto);

        commands.send(Commands::Persist(persistence_unit)).unwrap();
    }
}
