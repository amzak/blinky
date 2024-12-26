use std::{any::type_name, future::Future, mem};

use log::{debug, error, info};
use tokio::{
    select,
    sync::broadcast::{channel, Receiver, Sender},
};

use crate::{commands::Commands, events::Events};

pub struct BusSender {
    commands_sender: Sender<Commands>,
    events_sender: Sender<Events>,
}

pub struct MessageBus {
    sender: BusSender,
    commands_recv: Option<Receiver<Commands>>,
    events_recv: Option<Receiver<Events>>,
}

pub struct ContextStub {}

pub trait BusHandler<TContext> {
    fn event_handler(
        bus: &BusSender,
        context: &mut TContext,
        event: Events,
    ) -> impl Future<Output = ()> + Send;
    fn command_handler(
        bus: &BusSender,
        context: &mut TContext,
        command: Commands,
    ) -> impl Future<Output = ()> + Send;
}

impl MessageBus {
    pub fn clone(self: &MessageBus) -> Self {
        Self {
            sender: BusSender {
                commands_sender: self.sender.commands_sender.clone(),
                events_sender: self.sender.events_sender.clone(),
            },
            commands_recv: None,
            events_recv: None,
        }
    }
}

impl BusSender {
    pub fn send_cmd(&self, command: Commands) {
        self.commands_sender.send(command).unwrap();
    }

    pub fn send_event(&self, event: Events) {
        self.events_sender.send(event).unwrap();
    }
}

impl Clone for BusSender {
    fn clone(&self) -> Self {
        Self {
            commands_sender: self.commands_sender.clone(),
            events_sender: self.events_sender.clone(),
        }
    }
}

impl MessageBus {
    pub fn new() -> Self {
        let (commands_sender, commands_recv) = channel::<Commands>(20);
        let (events_sender, events_recv) = channel::<Events>(64);

        Self {
            sender: BusSender {
                commands_sender,
                events_sender,
            },
            commands_recv: Some(commands_recv),
            events_recv: Some(events_recv),
        }
    }

    #[inline]
    pub async fn handle<TContext, THandler>(mut bus: MessageBus, mut context: TContext) -> TContext
    where
        THandler: BusHandler<TContext>,
    {
        let handler_type = type_name::<THandler>();

        let size_of_context = mem::size_of::<TContext>();

        debug!("context {} bytes", size_of_context);

        info!("starting handle loop... {}", handler_type);

        if bus.commands_recv.is_none() {
            bus.commands_recv = Some(bus.sender.commands_sender.subscribe());
        };

        if bus.events_recv.is_none() {
            bus.events_recv = Some(bus.sender.events_sender.subscribe());
        };

        let commands_receiver = bus.commands_recv.as_mut().unwrap();
        let events_receiver = bus.events_recv.as_mut().unwrap();

        let mut break_loop = false;

        loop {
            break_loop = Self::handle_command_or_event::<TContext, THandler>(
                &bus.sender,
                commands_receiver,
                events_receiver,
                &mut context,
                handler_type,
            )
            .await;

            if break_loop {
                break;
            }
        }

        info!("done {}", handler_type);

        context
    }

    async fn handle_command_or_event<TContext, THandler>(
        sender: &BusSender,
        commands_receiver: &mut Receiver<Commands>,
        events_receiver: &mut Receiver<Events>,
        context: &mut TContext,
        handler_type: &str,
    ) -> bool
    where
        THandler: BusHandler<TContext>,
    {
        let mut break_loop = false;

        select! {
            command_res = commands_receiver.recv() => {
                match command_res {
                    Ok(command) => {

                        if matches!(command, Commands::StartDeepSleep) {
                            break_loop = true;
                        }

                        THandler::command_handler(sender, context, command).await;
                    },
                    Err(err) => {error!("{:?} {:?}", err, handler_type)},
                }
             }
             event_res = events_receiver.recv() => {
                match event_res {
                    Ok(event) => THandler::event_handler(sender, context, event).await,
                    Err(err) => {error!("{:?} {:?}", err, handler_type)},
                }
            }
        }

        return break_loop;
    }

    pub fn send_cmd(&self, command: Commands) {
        if let Err(err) = self.sender.commands_sender.send(command) {
            error!("{:?}", err);
        }
    }

    pub fn send_event(&self, event: Events) {
        if let Err(err) = self.sender.events_sender.send(event) {
            error!("{:?}", err);
        }
    }

    pub async fn wait_for(&mut self, target_event: Events) {
        let mut events_receiver = self.sender.events_sender.subscribe();

        info!("waiting for {:?}...", target_event);

        loop {
            let event_res = events_receiver.recv().await;
            match event_res {
                Ok(event) => {
                    error!("comparing events {:?} target {:?}", event, target_event);

                    if std::mem::discriminant(&event) == std::mem::discriminant(&target_event) {
                        break;
                    }
                }
                Err(err) => {
                    error!("waiting for {:?} {:?}", err, target_event)
                }
            }
        }

        info!("resuming after {:?}", target_event);
    }
}
