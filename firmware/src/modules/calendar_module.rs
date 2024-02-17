use std::collections::HashSet;

use crate::modules::reference_time::ReferenceTimeUtc;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset};

use blinky_shared::message_bus::{BusHandler, BusSender, MessageBus};
use blinky_shared::{
    calendar::CalendarEvent,
    error::Error,
    events::Events,
    persistence::{PersistenceUnit, PersistenceUnitKind},
};
use blinky_shared::{calendar::CalendarEventDto, commands::Commands};
pub struct CalendarModule {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarStateDto {
    pub version: i32,
    pub last_sync: ReferenceTimeUtc,
    pub events: Vec<CalendarEventDto>,
}

struct Context {
    calendar_events: HashSet<CalendarEvent>,
    now: Option<OffsetDateTime>,
    utc_offset: Option<UtcOffset>,
}

impl CalendarStateDto {
    pub fn new(events: Vec<CalendarEventDto>, last_sync: ReferenceTimeUtc) -> Self {
        Self {
            version: 2,
            events,
            last_sync,
        }
    }
}

impl BusHandler<Context> for CalendarModule {
    async fn event_handler(bus: &BusSender, context: &mut Context, event: Events) {
        match event {
            Events::TimeNow(time_now) => {
                context.now = Some(time_now);
            }
            Events::ReferenceCalendarEvent(reference_calendar_event) => {
                let replaced = context
                    .calendar_events
                    .replace(reference_calendar_event.clone());

                if replaced.is_some() {
                    info!("event updated {}", replaced.unwrap().id);
                }

                bus.send_event(Events::CalendarEvent(reference_calendar_event));
            }
            Events::BluetoothDisconnected => {
                if context.calendar_events.len() == 0 {
                    return;
                }

                Self::persist_events(
                    bus,
                    Vec::from_iter(context.calendar_events.iter().map(|x| x.clone())),
                    &context.now,
                )
                .await;

                info!("events persisted");
            }

            Events::Restored(unit) => {
                if !matches!(unit.kind, PersistenceUnitKind::CalendarSyncInfo) {
                    return;
                }

                if let Err(error) = unit.data {
                    error!("{}", error);
                    return;
                }

                if context.utc_offset.is_none() {
                    warn!("utc_offset is not set");
                    return;
                }

                Self::try_restore(
                    bus,
                    &mut context.calendar_events,
                    unit,
                    context.utc_offset.unwrap(),
                )
                .await;
            }
            _ => {}
        }
    }

    async fn command_handler(bus: &BusSender, context: &mut Context, command: Commands) {
        match command {
            Commands::SyncCalendar => {
                bus.send_event(Events::InSync(false));
            }
            Commands::SetTimezone(offset) => {
                context.utc_offset = Some(UtcOffset::from_whole_seconds(offset).unwrap());
                bus.send_cmd(Commands::Restore(PersistenceUnitKind::CalendarSyncInfo));
            }
            _ => {}
        }
    }
}

impl CalendarModule {
    pub async fn start(mut bus: MessageBus) {
        info!("starting...");

        let context = Context {
            calendar_events: HashSet::new(),
            now: None,
            utc_offset: None,
        };

        MessageBus::handle::<Context, Self>(bus, context).await;

        info!("done.");
    }

    async fn try_restore(
        bus: &BusSender,
        calendar_events: &mut HashSet<CalendarEvent>,
        posponed_restore: PersistenceUnit,
        utc_offset: UtcOffset,
    ) -> bool {
        let res: Result<CalendarStateDto, Error> = posponed_restore.deserialize();

        match res {
            Ok(calendar_info_restored) => {
                info!(
                    "calendar events restored: {}",
                    calendar_info_restored.events.len()
                );

                let calendar_events_restored = calendar_info_restored
                    .events
                    .into_iter()
                    .map(|x| CalendarEvent::new(x, utc_offset));

                for event in calendar_events_restored {
                    calendar_events.insert(event.clone());
                    bus.send_event(Events::CalendarEvent(event));
                }
            }
            Err(error) => {
                error!("{:?}", error);
                return false;
            }
        }

        return true;
    }

    async fn persist_events(
        commands: &BusSender,
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

        commands.send_cmd(Commands::Persist(persistence_unit));
    }
}
