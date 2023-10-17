//! # Actor Supervisor
//! This module contains the [`Supervisor`]. This struct contains an actor, alongside code dedicated to handling messages for the actor.

use alloc::{boxed::Box, sync::Arc};
use futures::FutureExt;

use crate::{FluxionParams, Actor, InvertedHandler, types::broadcast, ActorError, Executor};

use super::handle::LocalHandle;

/// # [`Supervisor`]
/// This struct wraps an actor, and is owned by a task which constantly receives messages over an asynchronous mpsc channel.
pub struct Supervisor<C: FluxionParams, A: Actor<C>> {
    /// The supervised actor
    actor: Arc<A>,
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
            actor: Arc::new(actor),
            messages,
            shutdown,
        }
    }

    /// Returns a handle for this supervisor
    #[must_use]
    pub fn handle(&self) -> LocalHandle<C, A> {
        LocalHandle {
            sender: self.messages.clone(),
        }
    }

    /// Internal function that runs the supervisor's main loop for receiving messages
    /// Returns any errors immediately, returns Ok(()) when the main loop terminates
    /// gracefully, most likely by a call to shutdown.
    /// 
    /// # Errors
    /// This function errors whenever one of the following occurs:
    /// - Receiving a message fails
    /// - Handling a message fails
    async fn tick(&mut self) -> Result<(), ActorError<A::Error>> {
        loop {
            futures::select_biased! {
                _ = self.shutdown.recv().fuse() => {
                    break;
                },
                mut next = self.messages.recv().fuse() => {
                    // Clone the actor as an Arc, allowing us to send it between threads
                    let actor = self.actor.clone();
                    
                    // Handle the message in a separate task
                    <C::Executor as Executor>::spawn(async move {
                        next.handle(&actor).await;
                    });
                }
            }
        }

        // Return Ok
        Ok(())
    }

    /// Runs the actor's entire lifecycle, returning any errors along the way
    pub async fn run(&mut self) -> Result<(), ActorError<Actor::Error>> {
        Ok(())
    }

    
}