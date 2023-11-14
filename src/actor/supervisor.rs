//! # Actor Supervisor
//! This module contains the [`Supervisor`], which wraps an actor and message channel, delegating received messages and handling the actor's lifecycle.

use alloc::{boxed::Box, sync::Arc};
use async_oneshot::Sender;
use maitake_sync::RwLock;
use crate::alloc::string::ToString;

use crate::{FluxionParams, Actor, InvertedHandler, ActorError, Executor, ActorContext, ActorId};
use super::{handle::LocalHandle, ActorControlMessage};

#[cfg(error_policy)]
use crate::handle_policy;

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
    #[cfg_attr(tracing, tracing::instrument(skip(self)))]
    async fn tick(&mut self) -> Result<Sender<()>, ActorError<A::Error>> {

        
        // A channel for receiving errors
        let error_channel = whisk::Channel::<ActorError<A::Error>>::new();
        
        // Get the ID as an owned type so we can use it from other tasks when needed.
        let id = self.context.get_id().0;

        crate::event!(tracing::Level::DEBUG, "[{id}] Began ticking.");

        loop {

            // Receive the next message
            let next = futures::select_biased! {
                err = futures::FutureExt::fuse(error_channel.recv()) => {
                    // If an error, log and return the error.
                    crate::event!(tracing::Level::INFO, error=err.to_string(), "[{id}] Received kill message. ");
                    Err(err)
                }
                next = futures::FutureExt::fuse(self.messages.recv()) => {
                    // If there is a message, log and return Ok
                    crate::event!(tracing::Level::TRACE, "[{id}] Received control message.");
                    Ok(next)
                }
            }?;

            // Clone the error channel
            let errors = error_channel.clone();

            

            match next {
                ActorControlMessage::Message(mut message) => {

                    crate::event!(tracing::Level::TRACE, "[{id}] Received message from {:?}", message.sender());

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

                        crate::event!(tracing::Level::TRACE, "[{id}] Spawned message handler.");

                        // If error policies are disabled, simulate them failing on all errors
                        #[cfg(not(error_policy))]
                        let res = match  message.handle(&context, &a).await {
                            Ok(v) => Ok(Ok::<_, ActorError<A::Error>>(v)),
                            Err(e) => Err(e),
                        };
                        #[cfg(error_policy)]
                        let res = handle_policy!(
                            async {
                                crate::event!(tracing::Level::TRACE, "[{id}] Running handle in error policy.");
                                message.handle(&context, &a).await
                            }.await, |_| { A::ERROR_POLICY },
                            (), ActorError<A::Error>
                        ).await;

                        // Handle errors
                        match res {
                            Ok(r) => {
                                // If this is Ok, it means that either the handler was successful,
                                // or that the error policy ignored the error.
                                // If the later is the case, we should log it if tracing is enabled
                                #[cfg(tracing)]
                                if let Err(e) = r {
                                    crate::event!(tracing::Level::WARN, error=e.to_string(), "[{id}] Error while handling message. Error ignored by policy.");
                                }
                            },
                            Err(e) => {
                                // Log this
                                crate::event!(tracing::Level::ERROR, error=e.to_string(), "[{id}] Error while handling messages. Policy dictates actor to be killed.");

                                // Any other errors should kill the actor
                                errors.send(e).await;
                            },
                        }
                    });
                },
                ActorControlMessage::Shutdown(s) => {
                    // If we should shutdown, return the acknowledgement channel
                    // so that we only acknowledge after all deinitialization has been run
                    crate::event!(tracing::Level::TRACE, "[{id}] Recieved (and fulfilling) shutdown request.");
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
    #[cfg_attr(tracing, tracing::instrument(skip(self)))]
    async fn run_internal(&mut self) -> Result<Sender<()>, ActorError<A::Error>> {

        // Get the actor's id
        let id = self.context.get_id().0;

        crate::event!(tracing::Level::INFO, "[{id}] Starting.");
        
        // Initialize the actor, handling error policy
        match {
            crate::event!(tracing::Level::TRACE, "[{id}] Running initialization.");

            let mut actor = self.actor.write().await;

            #[cfg(not(error_policy))]
            match actor.initialize(&self.context).await {
                Ok(v) => Ok(Ok::<_, ActorError<A::Error>>(v)),
                Err(e) => Err(e),
            }
            #[cfg(error_policy)]
            handle_policy!(
                async {
                    crate::event!(tracing::Level::TRACE, "[{id}] Running initialization in error policy.");
                    actor.initialize(&self.context).await
                }.await, |_| { A::ERROR_POLICY },
                (), ActorError<A::Error>
            ).await
        } {
            Ok(res) => {
                // If the contained result is an error, we should log it.
                #[cfg(tracing)]
                if let Err(e) = res {
                    crate::event!(tracing::Level::WARN, error=e.to_string(), "[{id}] Error during initialization. Error ignored by policy.");
                }
            },
            Err(e) => {
                // Log and error
                crate::event!(tracing::Level::ERROR, error=e.to_string(), "[{id}] Error during initialization. Policy dictates actor to be killed.");
                return Err(e);
            }
        };

        
        // Tick the actor's main loop
        crate::event!(tracing::Level::TRACE, actor=self.context.get_id().as_ref().to_string(), "[{id}] Entering main loop");
        let res = self.tick().await;

        // If there was an error while ticking the main loop, log it
        if let Err(e) = &res {
            crate::event!(tracing::Level::DEBUG, error=e.to_string(), "[{id}] Error while ticking. Deinitialization will be run.");
        }

        // Deinitialize the actor, handling error policy
        match {
            crate::event!(tracing::Level::TRACE, "[{id}] Running deinitialization.");

            let mut actor = self.actor.write().await;

            #[cfg(not(error_policy))]
            match actor.deinitialize(&self.context).await {
                Ok(v) => Ok(Ok::<_, ActorError<A::Error>>(v)),
                Err(e) => Err(e),
            }
            #[cfg(error_policy)]
            handle_policy!(
                async {
                    crate::event!(tracing::Level::TRACE, "[{id}] Running deinitialization in error policy.");
                    actor.deinitialize(&self.context).await
                }.await, |_| { A::ERROR_POLICY },
                (), ActorError<A::Error>
            ).await
        } {
            Ok(res) => {
                // If the contained result is an error, we should log it.
                #[cfg(tracing)]
                if let Err(e) = res {
                    crate::event!(tracing::Level::WARN, error=e.to_string(), "[{id}] Error during deinitialization. Error ignored by policy.");
                }
            },
            Err(e) => {
                // Log and error
                crate::event!(tracing::Level::ERROR, error=e.to_string(), "[{id}] Error during deinitialization. Policy dictates actor to be killed.");
                return Err(e);
            }
        };
        

        res
    }

    /// Runs the actor's entire lifecycle.
    /// 
    /// # Errors
    /// This function errors whenever one of the following occurs:
    /// - Receiving a message fails
    /// - Handling a message fails
    /// - Initialization, deinitialization, or cleanup errors.
    #[cfg_attr(tracing, tracing::instrument(skip(self)))]
    pub async fn run(&mut self) -> Result<(), ActorError<A::Error>> {

        // Get the actor's id
        let id = self.context.get_id().0;

        // Run the application
        let res = self.run_internal().await;

        // Run cleanup, providing the error if the result was an error
        match res {
            Err(e) => {
                // When there is an error, make sure to borrow it.
                crate::event!(tracing::Level::DEBUG, error=e.to_string(), "[{id}] Exited with error. Running cleanup.");
                self.actor.write().await.cleanup(&self.context, Some(&e)).await?;
                Err(e)
            },
            Ok(mut s) => {
                // Cleanup with no errors
                crate::event!(tracing::Level::TRACE, "[{id}] Exited gracefully. Running cleanup with no errors.");
                self.actor.write().await.cleanup(&self.context, None).await?;

                // Acknowledge shutdown, ignoring the result (too late to do anything now.)
                let _ = s.send(());

                Ok(())
            }
        }
    }
    
}