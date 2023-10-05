//! # Actor Supervisor
//! This module contains the [`Supervisor`]. This struct contains an actor, alongside code dedicated to handling messages for the actor.

use alloc::boxed::Box;

use crate::types::{params::SupervisorParams, message::Handler, errors::ActorError, actor::Actor};


/// # [`Supervisor`]
/// This struct wraps an actor, and is owned by a task which constantly receives messages over an asynchronous mpsc channel.
pub struct Supervisor<Params: SupervisorParams> {
    /// The supervised actor
    actor: Params::Actor,
    /// The message channel
    messages: whisk::Channel<Box<dyn Handler<Params::Actor>>>,
}

impl<Params: SupervisorParams> Supervisor<Params> {

    /// Creates a new supervisor
    pub fn new(actor: Params::Actor) -> Self {
        // Create a new whisk channel
        let messages = whisk::Channel::new();

        // Create the supervisor
        Self {
            actor,
            messages
        }
    }

    /// Returns a channel which can send to the supervisor
    pub fn channel(&self) -> whisk::Channel<Box<dyn Handler<Params::Actor>>> {
        self.messages.clone()
    }

    /// Ticks the supervisor once
    /// 
    /// # Errors
    /// This function errors whenever one of the following occurs:
    /// - Receiving a message fails
    /// - Handling a message fails
    pub async fn tick(&self) -> Result<(), ActorError<<Params::Actor as Actor>::Error>> {

        // Receive the next message from the receiver
        let mut next = self.messages.recv().await;

        // Handle the message
        next.handle(&self.actor).await?;

        // Return Ok
        Ok(())
    }
}