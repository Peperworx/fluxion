//! # Message Tyes
//! Contains types and traits related to messages.

// Needed by async_trait.
#[cfg(async_trait)]
use alloc::boxed::Box;

#[cfg(serde)]
use serde::{Deserialize, Serialize};

use super::{actor::Actor, errors::{ActorError, SendError}, Handle};


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

impl Message for () {
    type Response = ();
}

/// # [`MessageSender`]
/// This trait allows sending messages to an actor without knowing the actor's type.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait MessageSender<M: Message>: Send + Sync + 'static {
    /// Send a message to an actor, and wait for a response
    /// 
    /// # Errors
    /// Errors if no response was received.
    async fn request(&self, message: M) -> Result<M::Response, super::errors::SendError>;
}


/// # [`Handler`]
/// Not to be confused with `Handle`. This trait is implemented for [`MessageHandler`],
/// allowing Fluxion to get rid of the `M` generic, allowing many different types
/// of messages to be sent.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait Handler<A: Actor>: Send + Sync {
    async fn handle(&mut self, actor: &A) -> Result<(), ActorError<A::Error>>;
}

/// # [`MessageHandler`]
/// This struct is sent over the channel to an actor. This is used in combination with the [`Handler`]
/// trait to allow an actor to receive many different message types.
pub struct MessageHandler<M: Message> {
    /// The message being sent
    message: M,
    /// The response channel
    responder: async_oneshot::Sender<M::Response>,
}

impl<M: Message> MessageHandler<M> {
    /// Creates a new [`MessageHandler`] and oneshot response receiver channel.
    pub fn new(message: M) -> (Self, async_oneshot::Receiver<M::Response>) {
        let (responder, rx) = async_oneshot::oneshot();

        (Self {
            message, responder
        }, rx)
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<A: Actor + Handle<M>, M: Message> Handler<A> for MessageHandler<M> {
    async fn handle(&mut self, actor: &A) -> Result<(), ActorError<A::Error>> {
        // Handle the message
        let res = actor.message(&self.message).await?;

        // Send the response
        self.responder.send(res).or(Err(SendError::ResponseFailed))?;

        Ok(())
    }
}