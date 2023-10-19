//! # Actor Supervisor
//! This module contains the [`Supervisor`]. This struct contains an actor, alongside code dedicated to handling messages for the actor.

use alloc::{boxed::Box, sync::Arc};
use futures::FutureExt;
use maitake_sync::RwLock;

use crate::{FluxionParams, Actor, InvertedHandler, ActorError, Executor};

use super::handle::LocalHandle;

/// # [`Supervisor`]
/// This struct wraps an actor, and is owned by a task which constantly receives messages over an asynchronous mpsc channel.
pub struct Supervisor<C: FluxionParams, A: Actor<C>> {
    /// The supervised actor
    actor: Arc<RwLock<A>>,
    /// The message channel
    messages: whisk::Channel<Option<Box<dyn InvertedHandler<C, A>>>>,
}

impl<C: FluxionParams, A: Actor<C>> Supervisor<C, A> {

    /// Creates a new supervisor
    pub fn new(actor: A) -> Self {
        // Create a new whisk channel
        let messages = whisk::Channel::new();

        // Create the supervisor
        Self {
            actor: Arc::new(RwLock::new(actor)),
            messages,
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

            // Receive the next message
            let next = self.messages.recv().await;

            // None means that we should gracefully exit.
            let Some(mut next) = next else {
                break;
            };

            // Clone the actor as an Arc, allowing us to send it between threads
            let actor = self.actor.clone();
                    
            // Handle the message in a separate task
            <C::Executor as Executor>::spawn(async move {
                let a = actor.read().await;
                next.handle(&a).await;
            });
        }

        // Return Ok
        Ok(())
    }

    /// Internal function that runs the application's entire lifecycle, except for cleanup, which is handled by [`run`]
    /// 
    /// # Errors
    /// Passes along any errors from a failure to receive messages or any errors returned by the actor.
    async fn run_internal(&mut self) -> Result<(), ActorError<A::Error>> {
        
        // Initialize the actor
        self.actor.write().await.initialize().await?;

        // Tick the actor's main loop
        self.tick().await;

        // Deinitialize the actor
        self.actor.write().await.deinitialize().await?;

        Ok(())
    }

    /// Runs the actor's entire lifecycle/
    /// 
    /// # Errors
    /// Passes along any errors from a failure to receive messages or any errors returned by the actor.
    pub async fn run(&mut self) -> Result<(), ActorError<A::Error>> {

        // Run the application
        let res = self.run_internal().await;

        // Run cleanup, providing the error if the result was an error
        match res {
            Err(e) => {
                // When there is an error, make sure to borrow it.
                self.actor.write().await.cleanup(Some(&e)).await?;
                Err(e)
            },
            Ok(()) => {
                // Cleanup no errors
                self.actor.write().await.cleanup(None).await
            }
        }
    }
    
}