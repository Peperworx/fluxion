//! # Message
//! The [`Message`] trait encapsulates all Messages that can be sent between actors, including Notifications and Federated Messages.

use crate::actor::actor_ref::ActorRef;
use crate::actor::{wrapper::ActorWrapper, Actor, Handle};

use crate::error::FluxionError;

// Only used by async_trait
#[cfg(async_trait)]
use alloc::boxed::Box;

#[cfg(serde)]
use {
    alloc::vec::Vec,
    serde::{Deserialize, Serialize},
};

/// # Message
/// This trait is used to mark Messages. Notifications are just Messages with a response type of `()`.
/// By default, all Messages and their responses must be [`Send`] + [`Sync`] + [`'static`].
pub trait Message: Send + Sync + 'static {
    /// The message's response
    type Response: Send + Sync + 'static;
}

/// # `MessageSerializer`
/// This trait is used to simplify the serialization and deserialization of messages and their responses
#[cfg(serde)]
pub trait MessageSerializer {
    /// Deserialize a message
    ///
    /// # Errors
    /// This function should only ever error with a [`FluxionError::DeserializeError`].
    fn deserialize<T: for<'a> Deserialize<'a>, E>(message: Vec<u8>) -> Result<T, FluxionError<E>>;

    /// Serialize a message
    ///
    /// # Errors
    /// This function should only ever error with a [`FluxionError::SerializeError`].
    fn serialize<T: Serialize, E>(message: T) -> Result<Vec<u8>, FluxionError<E>>;
}

/// # `MessageHandler`
/// This is the struct that is actually sent over the channel to an actor and stores both a message and its responder.
/// This is primarilly to reduce repetitive code.
pub struct MessageHandler<M: Message> {
    /// The message
    message: M,
    /// The responder
    responder: async_oneshot::Sender<M::Response>,
}

impl<M: Message> MessageHandler<M> {
    /// Create a new [`MessageHandler`]
    pub fn new(message: M, responder: async_oneshot::Sender<M::Response>) -> Self {
        Self { message, responder }
    }
}

/// # Dispatcher
/// Enables an actor to receive many different message types over the same channel.
pub trait Dispatcher {
    /// This function is called whenever a message is to be dispatched to an [`ActorRef`] which implements [`Dispatch`]
    fn dispatch<M: Message>(&self, message: M)
    where
        Self: Sized;
}
impl<A: Actor + Handle<BaseMessage>, BaseMessage: Message> Dispatcher for ActorRef<A> {
    fn dispatch<M: Message + Into<BaseMessage>>(&self, message: M)
    where
        Self: Sized,
    {
        todo!()
    }
}
