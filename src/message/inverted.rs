//! # Inverted
//! Inverted message handlers allow the `handle` function to be called on the messages themselves, instead of the actor.
//! This allows various generics to be removed, and enables many different message types to be sent to the same actor.

use crate::{FluxionParams, Actor, ActorError, Message, SendError, Handler, Executor};

#[cfg(async_trait)]
use alloc::boxed::Box;

/// # [`InvertedHandler`]
/// Enables the `handle` function to be called on struct wrapping a message, removing the requirement
/// of knowing the message's type. 
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait InvertedHandler<C: FluxionParams, A: Actor<C>>: Send + Sync {
    async fn handle(&mut self, actor: &A) -> Result<(), ActorError<A::Error>>;
}

/// # [`InvertedMessage`]
/// This struct wraps a message, and is sent over the actor's channel as a `Box<dyn InvertedHandle<C, A>>`.
/// This removes the requirement for the actor to know the message's type, instead requiring that the message
/// sender know the actors type. A different trick allows us to then switch this back to only knowing the message's
/// type, but only after actor creation. This allows many different message types to be sent to the same actor.
pub struct InvertedMessage<M: Message> {
    /// The message being sent
    message: M,
    /// The response channel
    responder: async_oneshot::Sender<M::Response>,
}

impl<M: Message> InvertedMessage<M> {
    /// Creates a new [`MessageHandler`] and oneshot response receiver channel.
    pub fn new(message: M) -> (Self, async_oneshot::Receiver<M::Response>) {
        let (responder, rx) = async_oneshot::oneshot();

        (Self {
            message, responder
        }, rx)
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<C: FluxionParams, A: Handler<C, M>, M: Message> InvertedHandler<C, A> for InvertedMessage<M> {
    async fn handle(&mut self, actor: &A) -> Result<(), ActorError<A::Error>> {
        <C::Executor as Executor>::spawn(async {
            // Handle the message
            let res = actor.message(&self.message).await;

            let Ok(res) = res else {
                return;
            };

            // Send the response
            self.responder.send(res).or(Err(SendError::ResponseFailed));
        });
        Ok(())
    }
}