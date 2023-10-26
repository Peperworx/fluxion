//! # Message Tyes
//! Contains types and traits related to messages.

pub mod inverted;


// Needed by async_trait.
#[cfg(async_trait)]
use alloc::boxed::Box;

use alloc::vec::Vec;

#[cfg(serde)]
use serde::{Deserialize, Serialize};

use crate::{Actor, FluxionParams, ActorError, ActorContext, ActorId};


/// # Message
/// This trait is used to mark Messages. Notifications are just Messages with a response type of `()`.
/// By default, all Messages and their responses must be [`Send`] + [`Sync`] + [`'static`].
pub trait Message: Send + Sync + 'static {
    /// The message's response
    type Response: Send + Sync + 'static;
}

impl Message for () {
    type Response = ();
}


/// # Handle
/// Actors MAY implement this trait to handle messages or notifications.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait Handler<C: FluxionParams, M: Message>: Actor<C> {
    async fn message(
        &self,
        context: &ActorContext<C>,
        message: &M
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
    /// Errors if no response was received.
    async fn request(&self, message: M) -> Result<M::Response, crate::types::errors::SendError>;
}


/// # [`ForeignMessage`]
/// Contains data for foreign messages that are used in their routing and resolution.
#[cfg(foreign)]
pub struct ForeignMessage {
    /// The message's target; the actor that the message is being sent to.
    pub target: ActorId,
    /// The source of the message; the actor that is sending the message.
    /// If the message is sent outside of the actor, the system will be the originating system,
    /// while the actor field will be empty.
    pub source: ActorId,
    /// The contents of the message
    pub message: Vec<u8>,
    /// The channel to respond to the message over.
    pub responder: async_oneshot::Sender<Vec<u8>>,
}
