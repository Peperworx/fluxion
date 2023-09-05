//! # Message
//! The [`Message`] trait encapsulates all Messages that can be sent between actors, including Notifications and Federated Messages.

use crate::error::FluxionError;

// Only used by async_trait
#[cfg(async_trait)]
use alloc::boxed::Box;

#[cfg(serde)]
pub mod serializer;

#[cfg(foreign)]
pub mod foreign;

/// # Message
/// This trait is used to mark Messages. Notifications are just Messages with a response type of `()`.
/// By default, all Messages and their responses must be [`Send`] + [`Sync`] + [`'static`].
pub trait Message: Send + Sync + 'static {
    /// The message's response
    type Response: Send + Sync + 'static;

    /// The custom error type that might be returned by the message
    type Error: Send + Sync + 'static;
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
    pub fn respond(&mut self, response: M::Response) -> Result<(), FluxionError<M::Error>> {
        self.responder
            .send(response)
            .or(Err(FluxionError::ResponseFailed))
    }
}

/// # `MessageGenerics`
/// This trait uses a really weird abstraction to fit multiply types in the same generic.
/// It is used as an extension of [`SupervisorGenerics`]
/// This is used by both the [`ActorSupervisor`] and the [`ActorRef`], and is an extension
pub trait MessageGenerics {
    /// The primary message type of this actor
    type Message: Message;

    /// If notifications are enabled, this is the message type of the notification
    #[cfg(notification)]
    type Notification: Message;

    /// If federated messages are enabled, this is the message type of the federated messages
    #[cfg(federated)]
    type Federated: Message;
}
