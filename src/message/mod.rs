//! # Message
//! The [`Message`] trait encapsulates all Messages that can be sent between actors, including Notifications and Federated Messages.

use crate::actor::{Handle, ActorContext, Actor, wrapper::ActorWrapper};


use alloc::boxed::Box;

#[cfg(serde)]
use {
    serde::{Serialize, Deserialize},
    crate::error::FluxionError,
    alloc::vec::Vec
};


/// # Message
/// This trait is used to mark Messages. Notifications are just Messages with a response type of `()`.
/// By default, all Messages and their responses must be [`Send`] + [`Sync`] + [`'static`].
pub trait Message: Send + Sync + 'static {
    /// The message's response
    type Response: Send + Sync + 'static;
}



/// # MessageSerializer
/// This trait is used to simplify the serialization and deserialization of messages and their responses
#[cfg(serde)]
pub trait MessageSerializer {

    /// Deserialize a message
    fn deserialize<T: for<'a> Deserialize<'a>, E>(message: Vec<u8>) -> Result<T, FluxionError<E>>;

    /// Serialize a message
    fn serialize<T: Serialize, E>(message: T) -> Result<Vec<u8>, FluxionError<E>>;
}


/// # MessageHandler
/// This is the struct that is actually sent over the channel to an actor and stores both a message and its responder.
/// This is used in combination with the [`Handler`] trait to allow an actor to receive many different message types locally.
pub struct MessageHandler<M: Message> {
    /// The message
    message: M,
    /// The responder
    responder: async_oneshot::Sender<M::Response>,
}

impl<M: Message> MessageHandler<M> {
    /// Create a new MessageHandler
    pub fn new(message: M, responder: async_oneshot::Sender<M::Response>) -> Self {
        Self { message, responder }
    }
}


/// # Handler
/// This trait is implemented for [`MessageHandler`], allowing Fluxion to get rid of the `M` generic. This allows sending many different types of messages
/// to an Actor.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait Handler<A: Actor>: Send + Sync + 'static {
    /// Handle the message
    async fn handle(&mut self, actor: &mut ActorWrapper<A>) -> Result<(), FluxionError<A::Error>>;
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<A: Handle<M>, M: Message> Handler<A> for MessageHandler<M> {
    async fn handle(&mut self, actor: &mut ActorWrapper<A>) -> Result<(), FluxionError<A::Error>> {

        // Call the handler.
        let res = actor.dispatch(&self.message).await?;

        // Send the response
        self.responder.send(res).or(Err(FluxionError::ResponseFailed))?;

        Ok(())
    }
}