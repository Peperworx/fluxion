//! # [`ActorWrapper`]
//! Wraps an implementor of [`Actor`], enabling the dispatch of messages, as well as handling actor context.

use fluxion_error::ActorError;
use fluxion_message::Message;
use fluxion_types::{Actor, ActorContext, Handle};

/// # `ActorWrapper`
/// This struct wraps an implementor of [`Actor`], enabling the dispatch of messages, as well as handling actor context.
///
/// ## Multiple Message Types
/// While actors MAY implement handlers for multiple message types, however only one will work as a foreign message.
pub struct ActorWrapper<A: Actor> {
    /// The actual wrapped actor
    actor: A,
    /// The actor's execution context, which contains its reference to the system
    context: ActorContext,
}

impl<A: Actor> ActorWrapper<A> {
    /// Create a new [`ActorWrapper`]
    pub fn new(actor: A) -> Self {
        Self {
            actor,
            context: ActorContext,
        }
    }

    /// Dispatch a message to the contained actor.
    ///
    /// # Errors
    /// Returns any error return by the actor's message handler.
    pub async fn dispatch<M: Message>(
        &mut self,
        message: &M,
    ) -> Result<M::Response, ActorError<A::Error>>
    where
        A: Handle<M>,
    {
        self.actor.message(message, &self.context).await
    }

    /// Run actor initialization
    ///
    /// # Errors
    /// Returns any error returned by the actor's initialization logic.
    pub async fn initialize(&mut self) -> Result<(), ActorError<A::Error>> {
        self.actor.initialize(&self.context).await
    }

    /// Run actor deinitialization
    ///
    /// # Errors
    /// Returns any error returned by the actor's deinitialization logic.
    pub async fn deinitialize(&mut self) -> Result<(), ActorError<A::Error>> {
        self.actor.deinitialize(&self.context).await
    }

    /// Run actor cleanup
    ///
    /// # Errors
    /// Returns any error returned by the actor's cleanup logic.
    pub async fn cleanup(
        &mut self,
        error: Option<ActorError<A::Error>>,
    ) -> Result<(), ActorError<A::Error>> {
        self.actor.cleanup(error).await
    }
}
