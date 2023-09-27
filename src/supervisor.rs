//! # Actor Supervisor
//! This module contains the [`Supervisor`]. This struct contains an actor, alongside code dedicated to handling messages for the actor.

use alloc::{sync::Arc, boxed::Box};

use crate::types::{params::SupervisorParams, message::{Handler, Message}, errors::ActorError, actor::Actor};


/// # [`Supervisor`]
/// This struct wraps an actor, and is owned by a task which constantly receives messages over an asynchronous mpsc channel.
pub struct Supervisor<Params: SupervisorParams> {
    /// The supervised actor
    actor: Params::Actor,
    /// The message channel
    messages: thingbuf::mpsc::Receiver<Option<Box<dyn Handler<Params::Actor>>>>,
}

impl<Params: SupervisorParams> Supervisor<Params> {

    /// Creates a new supervisor and message channel, returning both
    pub fn new(actor: Params::Actor)  {
        

        todo!()
    }

    /// Ticks the supervisor once
    /// 
    /// # Errors
    /// This function errors whenever one of the following occurs:
    /// - Receiving a message fails
    /// - Handling a message fails
    pub async fn tick(&self) -> Result<(), ActorError<<Params::Actor as Actor>::Error>> {

        // Receive the next message from the receiver
        let next = self.messages.recv().await;

        // If it failed, then error
        let Some(next) = next else {
            return Err(ActorError::MessageReceiveError)
        };

        // Properly sent messages will always be Some, as the Option is just to
        // satisfy thingbuf's default requirement.
        let Some(mut next) = next else {
            // But we don't want this to crash the actor
            return Ok(());
        };

        // Handle the message
        next.handle(&self.actor).await?;

        // Return Ok
        Ok(())
    }
}