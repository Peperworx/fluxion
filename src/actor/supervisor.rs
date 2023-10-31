//! # Actor Supervisor
//! This module contains the [`Supervisor`], which wraps an actor and message channel, delegating received messages and handling the actor's lifecycle.

use alloc::{boxed::Box, sync::Arc};
use async_oneshot::Sender;
use maitake_sync::RwLock;

use crate::{FluxionParams, Actor, InvertedHandler, ActorError, Executor, ActorContext, ActorId};
use super::{handle::LocalHandle, ActorControlMessage};

/// # [`Supervisor`]
/// This struct wraps an actor, and is owned by a task which constantly receives messages over an asynchronous mpsc channel.
/// These messages are delegated to the actor.
pub struct Supervisor<C: FluxionParams, A: Actor<C>> {
    /// The supervised actor
    actor: Arc<RwLock<A>>,
    /// The message channel
    messages: whisk::Channel<ActorControlMessage<Box<dyn InvertedHandler<C, A>>>>,
    /// The actor's context
    context: Arc<ActorContext<C>>,
}

impl<C: FluxionParams, A: Actor<C>> Supervisor<C, A> {

    /// Creates a new supervisor
    pub fn new(actor: A, context: ActorContext<C>) -> Self {
        // Create a new whisk channel
        let messages = whisk::Channel::new();

        // Create the supervisor
        Self {
            actor: Arc::new(RwLock::new(actor)),
            messages,
            context: Arc::new(context),
        }
    }

    /// Returns a handle for this supervisor, owned by the provided actor id.
    #[must_use]
    pub fn handle(&self, owner: Option<ActorId>) -> LocalHandle<C, A> {
        LocalHandle {
            sender: self.messages.clone(),
            owner,
            target: self.context.get_id()
        }
    }

    /// Internal function that runs the supervisor's main loop for receiving messages
    /// Returns any errors immediately, and returns a shutdown acknowledgement channel
    /// whenever the actor exits gracefully.
    /// 
    /// # Errors
    /// This function errors whenever one of the following occurs:
    /// - Receiving a message fails
    /// - Handling a message fails
    async fn tick(&mut self) -> Result<Sender<()>, ActorError<A::Error>> {
        loop {

            // Receive the next message
            let next = self.messages.recv().await;

            match next {
                ActorControlMessage::Message(mut message) => {
                    // Clone the actor as an Arc, allowing us to send it between threads
                    let actor = self.actor.clone();
                    
                    // Clone the context as an arc too
                    let context = self.context.clone();

                    // Handle the message in a separate task
                    <C::Executor as Executor>::spawn(async move {
                        let a = actor.read().await;
                        if message.handle(&context, &a).await.is_err() {
                            todo!("Error handling")
                        }
                    });
                },
                ActorControlMessage::Shutdown(s) => {
                    // If we should shutdown, return the acknowledgement channel
                    // so that we only acknowledge after all deinitialization has been run
                    return Ok(s);
                },
            }

            
        }
    }

    /// Internal function that runs the application's entire lifecycle, except for cleanup, which is handled by [`Self::run`]
    /// Returns the shutdown acknowledgement channel when exiting gracefully.
    /// 
    /// # Errors
    /// Passes along any errors from a failure to receive messages or any errors returned by the actor.
    async fn run_internal(&mut self) -> Result<Sender<()>, ActorError<A::Error>> {
        
        // Initialize the actor
        self.actor.write().await.initialize(&self.context).await?;

        // Tick the actor's main loop
        let res = self.tick().await;

        // Deinitialize the actor
        self.actor.write().await.deinitialize(&self.context).await?;

        res
    }

    /// Runs the actor's entire lifecycle.
    /// 
    /// # Errors
    /// This function errors whenever one of the following occurs:
    /// - Receiving a message fails
    /// - Handling a message fails
    /// - Initialization, deinitialization, or cleanup errors.
    pub async fn run(&mut self) -> Result<(), ActorError<A::Error>> {

        // Run the application
        let res = self.run_internal().await;

        // Run cleanup, providing the error if the result was an error
        match res {
            Err(e) => {
                // When there is an error, make sure to borrow it.
                self.actor.write().await.cleanup(&self.context, Some(&e)).await?;
                Err(e)
            },
            Ok(mut s) => {
                // Cleanup with no errors
                self.actor.write().await.cleanup(&self.context, None).await?;

                // Acknowledge shutdown, ignoring the result (too late to do anything now.)
                let _ = s.send(());

                Ok(())
            }
        }
    }
    
}