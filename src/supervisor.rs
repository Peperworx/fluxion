//! # Actor Supervisor
//! This module contains the [`Supervisor`]. This struct contains an actor, alongside code dedicated to handling messages for the actor.

use alloc::{sync::Arc, boxed::Box};

use crate::types::{params::SupervisorParams, message::{MessageHandler, Message}, errors::ActorError, actor::Actor};


/// # [`Supervisor`]
/// This struct wraps an actor, and is owned by a task which constantly receives messages over an asynchronous mpsc channel.
pub struct Supervisor<A: Actor> {
    /// The supervised actor
    actor: A,
    /// The message channel
    messages: flume::Receiver<Box<dyn MessageHandler<A>>>,
}

impl<A: Actor> Supervisor<A> {

    /// Creates a new supervisor and message channel, returning both
    pub fn new(actor: A) -> (Self, flume::Sender<Box<dyn MessageHandler<A>>>) {
        let (tx, messages) = flume::unbounded();

        (Self {
            actor, messages
        }, tx)
    }

    /// Ticks the supervisor once
    /// 
    /// # Errors
    /// This function errors whenever one of the following occurs:
    /// - Receiving a message fails
    /// - Handling a message fails
    pub async fn tick(&self) -> Result<(), ActorError<A::Error>> {

        // Receive the next message from the receiver
        let next = self.messages.recv_async().await;

        // If it failed, then error
        let Ok(mut next) = next else {
            return Err(ActorError::MessageReceiveError)
        };

        // Handle the message
        next.handle(&self.actor).await?;

        // Return Ok
        Ok(())
    }
}