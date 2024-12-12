use blinky_shared::calendar::CalendarEventKey;
use blinky_shared::domain::ReferenceTimeUtc;
use blinky_shared::reminders::Reminder;
use itertools::Itertools;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use time::{Duration, OffsetDateTime, UtcOffset};

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
    update_events: HashSet<CalendarEvent>,
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
                handle_event_update(context, &reference_calendar_event);

                bus.send_event(Events::CalendarEvent(reference_calendar_event));
            }
            Events::ReferenceCalendarEventUpdatesBatch(batch) => {
                for event in batch.as_slice() {
                    handle_event_update(context, event);
                }

                bus.send_event(Events::CalendarEventsBatch(batch));
            }
            Events::ReferenceCalendarEventDropsBatch(batch) => {
                handle_events_drop(context, batch.as_slice());

                bus.send_event(Events::DropCalendarEventsBatch(batch));
            }
            Events::InSync(true) => {
                if context.update_events.len() == 0 {
                    return;
                }

                Self::persist_events(bus, context).await;

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
                    &mut context.update_events,
                    unit,
                    context.utc_offset.unwrap(),
                )
                .await;

                Self::set_reminders(context, bus);
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
                let utc_offset = UtcOffset::from_whole_seconds(offset).unwrap();
                context.utc_offset = Some(utc_offset);

                if context.now.is_some() {
                    context.now = Some(context.now.unwrap().replace_offset(utc_offset));
                }

                bus.send_cmd(Commands::Restore(PersistenceUnitKind::CalendarSyncInfo));
            }
            _ => {}
        }
    }
}

fn handle_event_update(context: &mut Context, reference_calendar_event: &CalendarEvent) {
    let replaced = context
        .update_events
        .insert(reference_calendar_event.clone());

    if replaced {
        info!("event updated {}", reference_calendar_event.id);
    }
}

fn handle_events_drop(context: &mut Context, events_keys: &[CalendarEventKey]) {
    let mut set: HashSet<CalendarEventKey> = HashSet::new();

    for event_key in events_keys.iter() {
        set.insert(event_key.clone());
    }

    context.update_events.retain(|x| !set.contains(&x.key()));

    if !set.is_empty() {
        info!("removed {} events", set.len());
    }
}

impl CalendarModule {
    pub async fn start(bus: MessageBus) {
        info!("starting...");

        let context = Context {
            update_events: HashSet::new(),
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
        let res: Result<CalendarStateDto, Error> = posponed_restore.deserialize().await;

        match res {
            Ok(calendar_info_restored) => {
                info!(
                    "calendar events restored: {}",
                    calendar_info_restored.events.len()
                );

                let chunk_size = 5;

                let calendar_events_restored = calendar_info_restored
                    .events
                    .iter()
                    .map(|x| CalendarEvent::new(x, utc_offset))
                    .chunks(chunk_size);

                for chunk in calendar_events_restored.into_iter() {
                    let mut batch = Vec::with_capacity(chunk_size);

                    for event in chunk.into_iter() {
                        calendar_events.insert(event.clone());
                        batch.push(event);
                    }

                    bus.send_event(Events::CalendarEventsBatch(Arc::new(batch)));
                }
            }
            Err(error) => {
                error!("{:?}", error);
                return false;
            }
        }

        return true;
    }

    async fn persist_events(bus: &BusSender, context: &mut Context) {
        let now = context.now;

        if now.is_none() {
            return;
        }

        let now = now.unwrap();

        context.update_events.retain(|x| x.end >= now);

        Self::set_reminders(context, bus);

        let calendar_events = context.update_events.iter();

        let event_keys: Vec<CalendarEventKey> = calendar_events
            .map(|x| CalendarEventKey(x.kind, x.id))
            .collect();

        let calendar_events = context.update_events.iter();

        let dtos: Vec<CalendarEventDto> = calendar_events
            .into_iter()
            .map(|x| CalendarEventDto::from(x))
            .collect();

        let calendar_state_dto = CalendarStateDto::new(dtos, now.into());
        let persistence_unit =
            PersistenceUnit::new(PersistenceUnitKind::CalendarSyncInfo, &calendar_state_dto);

        bus.send_cmd(Commands::Persist(persistence_unit));
        bus.send_event(Events::PersistedCalendarEvents(Arc::new(event_keys)))
    }

    fn set_reminders(context: &mut Context, bus: &BusSender) {
        let now = context.now.unwrap();

        let reminders: Vec<_> = context
            .update_events
            .iter()
            .filter(|x| x.start >= now && x.end - x.start < Duration::days(1))
            .flat_map(|x| {
                return vec![
                    Reminder {
                        event_id: x.id,
                        kind: blinky_shared::reminders::ReminderKind::Notification,
                        remind_at: x.start - Duration::minutes(10),
                    },
                    Reminder {
                        event_id: x.id,
                        kind: blinky_shared::reminders::ReminderKind::Event,
                        remind_at: x.start,
                    },
                ];
            })
            .sorted_by(|x, y| Ord::cmp(&x.remind_at, &y.remind_at))
            .collect();

        bus.send_cmd(Commands::SetReminders(reminders));
    }
}
