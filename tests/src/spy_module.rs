use std::slice::Iter;

use blinky_shared::{
    commands::Commands,
    events::Events,
    message_bus::{BusHandler, BusSender, MessageBus},
};

pub struct SpyResult {
    pub events: Vec<Events>,
}

#[derive(Clone)]
struct Context {
    pub events: Vec<Events>,
    stopEvent: Events,
}

pub struct SpyModule {
    result: SpyResult,
}

impl BusHandler<Context> for SpyModule {
    async fn event_handler(_bus: &BusSender, context: &mut Context, event: Events) {
        let mut stop = false;

        if std::mem::discriminant(&event) == std::mem::discriminant(&context.stopEvent) {
            stop = true;
        }

        context.events.push(event);

        if stop {
            _bus.send_cmd(Commands::StartDeepSleep);
        }
    }

    async fn command_handler(_bus: &BusSender, _context: &mut Context, _command: Commands) {}
}

impl SpyModule {
    pub fn new() -> SpyModule {
        return SpyModule {
            result: SpyResult { events: vec![] },
        };
    }

    pub async fn start(&mut self, bus: MessageBus, stopEvent: Events) {
        let context = Context {
            events: vec![],
            stopEvent,
        };

        let context = MessageBus::handle::<Context, Self>(bus, context).await;

        self.result.events = context.events;
    }

    pub fn get_result(&self) -> Iter<Events> {
        let iter = self.result.events.iter();
        return iter;
    }
}
