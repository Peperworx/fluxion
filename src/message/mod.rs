//! # Message
//! Contains types and traits related to messages.

pub mod inverted;

pub mod event;

#[cfg(foreign)]
pub mod foreign;

// Needed by async_trait.
#[cfg(async_trait)]
use alloc::boxed::Box;


use crate::{Actor, FluxionParams, ActorError, ActorContext, Event,};


/// # [`Message`]
/// This trait is used to mark Messages. Notifications are just Messages with a response type of `()`.
/// By default, all Messages and their responses must be [`Send`] + [`Sync`] + [`'static`].
pub trait Message: Send + Sync + 'static {
    /// The message's response
    type Response: Send + Sync + 'static;
}

impl Message for () {
    type Response = ();
}


/// # [`Handler`]`
/// Actors may implement this trait to handle different message.
/// This trait can be implemented more than once for a given actor, and if the actor's type is known,
/// more than one of the message types can be used.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait Handler<C: FluxionParams, M: Message>: Actor<C> {
    async fn message(
        &self,
        context: &ActorContext<C>,
        message: &Event<M>
    ) -> Result<M::Response, ActorError<Self::Error>>;
}



/// # [`MessageSender`]
/// This trait allows sending messages to an actor without knowing the actor's type.
/// This reverses the inversion done in [`inverted`], while still allowing different message types to be used.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait MessageSender<M: Message>: Send + Sync + 'static {
    /// Send a message to an actor, and wait for a response
    /// 
    /// # Errors
    /// Returns [`SendError::NoResponse`] if no response was received.
    async fn request(&self, message: M) -> Result<M::Response, crate::types::errors::SendError>;
}


