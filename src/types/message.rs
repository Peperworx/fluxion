//! # Message Tyes
//! Contains types and traits related to messages.

// Needed by async_trait.
#[cfg(async_trait)]
use alloc::boxed::Box;

#[cfg(serde)]
use serde::{Deserialize, Serialize};

use super::{actor::Actor, errors::ActorError, Handle};


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

/// # MessageHandler
/// This trait is used to wrap a struct, erasing the generic for a message type.
/// This allows actors to respond to a message without knowing which message type was used.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait MessageHandler<A: Actor> {
    async fn handle(&mut self, actor: &A) -> Result<(), ActorError<A::Error>>;
}

/// # [`MessageWrapper`]
/// This struct wraps a message, and implements [`MessageHandler`]
pub struct MessageWrapper<M: Message> {
    /// The actual message
    message: M,
    /// The response channel
    responder: async_oneshot::Sender<M::Response>,
}

impl<M: Message> MessageWrapper<M> {
    pub fn new(message: M) -> (Self, async_oneshot::Receiver<M::Response>) {
        let (responder, rx) = async_oneshot::oneshot();

        (Self {
            message, responder
        }, rx)
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<A: Actor + Handle<M>, M: Message> MessageHandler<A> for MessageWrapper<M> {
    async fn handle(&mut self, actor: &A) -> Result<(), ActorError<A::Error>> {
        // Handle the message
        let res = actor.message(&self.message).await?;

        // Send the response
        self.responder.send(res).or(Err(ActorError::ResponseFailed))?;

        Ok(())
    }
}