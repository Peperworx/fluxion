use std::f32::consts::E;

use async_trait::async_trait;



pub struct Context;

#[derive(Debug)]
pub struct ActorError;

pub trait ActorMessage {
    type Response;
}

pub trait SystemEvent: Send + Sync + 'static {}

impl<T> SystemEvent for T where T: Send + Sync + 'static {}

#[async_trait]
pub trait Actor: Send + Sync + 'static {
    /// The message sent to the actor
    type Message: ActorMessage;

    /// The notify type that is published by the system
    type Notify;

    /// The Dynamic type that is published by the system
    type Dynamic;

    /// Called when the actor is started
    async fn initialize(&mut self, context: Context) -> Result<(), ActorError>;

    /// Called when the actor is stopped
    async fn deinitialize(&mut self, context: Context) -> Result<(), ActorError>;

    /// Called when the actor recieves a notify
    async fn notify(&mut self, context: Context, notify: Self::Notify) -> Result<(), ActorError>;

    /// Called when the actor recieves a message
    async fn message(&mut self, context: Context, message: Self::Message) -> Result<<Self::Message as ActorMessage>::Response, ActorError>;
}

#[async_trait]
pub trait Handler<E: SystemEvent> {
    /// Called when the actor recieves an event
    async fn event(&mut self, context: Context, event: E) -> Result<(), ActorError>;
}

