//! # Actor Supervisor
//! This module contains the [`Supervisor`]. This struct contains an actor, alongside code dedicated to handling messages for the actor.

use alloc::boxed::Box;

use crate::{FluxionParams, Actor, InvertedHandler, types::broadcast, ActorError};

use super::handle::LocalHandle;

/// # [`Supervisor`]
/// This struct wraps an actor, and is owned by a task which constantly receives messages over an asynchronous mpsc channel.
pub struct Supervisor<C: FluxionParams, A: Actor<C>> {
    /// The supervised actor
    actor: A,
    /// The message channel
    messages: whisk::Channel<Box<dyn InvertedHandler<C, A>>>,
    /// The shutdown channel
    shutdown: broadcast::Receiver<()>,
}

impl<C: FluxionParams, A: Actor<C>> Supervisor<C, A> {

    /// Creates a new supervisor
    pub fn new(actor: A, shutdown: broadcast::Receiver<()>) -> Self {
        // Create a new whisk channel
        let messages = whisk::Channel::new();

        // Create the supervisor
        Self {
            actor,
            messages,
            shutdown,
        }
    }

    /// Returns a handle for this supervisor
    pub fn handle(&self) -> LocalHandle<C, A> {
        LocalHandle {
            sender: self.messages.clone(),
        }
    }

    

    /// Ticks the supervisor once
    /// 
    /// # Errors
    /// This function errors whenever one of the following occurs:
    /// - Receiving a message fails
    /// - Handling a message fails
    pub async fn tick(&self) -> Result<(), ActorError<A::Error>> {

        // Receive the next message from the receiver
        let mut next = self.messages.recv().await;

        // Handle the message
        next.handle(&self.actor).await?;

        // Return Ok
        Ok(())
    }
}