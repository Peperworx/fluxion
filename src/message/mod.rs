//! # Message
//! The [`Message`] trait encapsulates all Messages that can be sent between actors, including Notifications and Federated Messages.

#[cfg(serde)]
use serde::{Deserialize, Serialize};

use crate::error::MessageError;

#[cfg(serde)]
pub mod serializer;

#[cfg(foreign)]
pub mod foreign;

/// # Message
/// This trait is used to mark Messages. Notifications are just Messages with a response type of `()`.
/// By default, all Messages and their responses must be [`Send`] + [`Sync`] + [`'static`].
#[cfg(not(serde))]
pub trait Message: Send + Sync + 'static {
    /// The message's response
    type Response: Send + Sync + 'static;
}
#[cfg(serde)]
pub trait Message: Serialize + for<'a> Deserialize<'a> + Send + Sync + 'static {
    /// The message's response
    type Response: Serialize + for<'a> Deserialize<'a> + Send + Sync + 'static;
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

    /// Respond to the message
    ///
    /// # Errors
    /// This function may return an error due to a closed channel. This error is unrecoverable.
    pub fn respond(&mut self, response: M::Response) -> Result<(), MessageError> {
        self.responder
            .send(response)
            .or(Err(MessageError::ResponseFailed))
    }

    /// Returns the contained message
    pub fn message(&self) -> &M {
        &self.message
    }
}
