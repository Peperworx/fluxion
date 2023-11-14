//! # Actor Supervisor
//! This module contains the [`Supervisor`], which wraps an actor and message channel, delegating received messages and handling the actor's lifecycle.

use alloc::{boxed::Box, sync::Arc};
use async_oneshot::Sender;
use maitake_sync::RwLock;
use crate::alloc::string::ToString;

use crate::{FluxionParams, Actor, InvertedHandler, ActorError, Executor, ActorContext, ActorId, handle_policy};
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

        // A channel for receiving errors
        let error_channel = whisk::Channel::<ActorError<A::Error>>::new();
        
        // Get the ID as an owned type so we can use it from other tasks when needed.
        let id = self.context.get_id().0;

        loop {

            // Receive the next message
            let next = futures::select_biased! {
                err = futures::FutureExt::fuse(error_channel.recv()) => {
                    // If an error, return the error.
                    Err(err)
                }
                next = futures::FutureExt::fuse(self.messages.recv()) => {
                    // If there is a message, return Ok
                    Ok(next)
                }
            }?;

            // Clone the error channel
            let errors = error_channel.clone();

            

            match next {
                ActorControlMessage::Message(mut message) => {
                    // Clone the actor as an Arc, allowing us to send it between threads
                    let actor = self.actor.clone();
                    
                    // Clone the context as an arc too
                    let context = self.context.clone();

                    // Clone the id to use in the task
                    let id = id.clone();

                    // Handle the message in a separate task
                    <C::Executor as Executor>::spawn(async move {
                        // Lock the actor
                        let a = actor.read().await;

                        // If error policies are disabled, simulate them failing on all errors
                        #[cfg(not(error_policy))]
                        let res = match  message.handle(&context, &a).await {
                            Ok(v) => Ok(Ok(v)),
                            Err(e) => Err(e),
                        };
                        #[cfg(error_policy)]
                        let res = handle_policy!(
                            message.handle(&context, &a).await, |_| { A::ERROR_POLICY },
                            (), ActorError<A::Error>
                        ).await;

                        // Handle errors
                        match res {
                            Ok(r) => {
                                // If this is Ok, it means that either the handler was successful,
                                // or that the error policy ignored the error.
                                // If the later is the case, we should log it if tracing is enabled
                                if let Err(e) = r {
                                    crate::event!(tracing::Level::WARN, actor=id.as_ref().to_string(), error=e.to_string(), "Error while handling message ignored by policy.");
                                }
                            },
                            Err(e) => {
                                // Any other errors should kill the actor
                                errors.send(e).await;
                            },
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