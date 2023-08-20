//! # ActorWrapper
//! Wraps an implementor of [`Actor`], enabling the dispatch of messages, as well as handling actor context.



use crate::message::Message;
use crate::error::FluxionError;


use super::{ActorContext, Actor, Handle};

/// # ActorWrapper
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
    pub async fn dispatch<M: Message>(&mut self, message: &M) -> Result<M::Response, FluxionError<A::Error>> where A: Handle<M> {
        self.actor.message(&message, &mut self.context).await
    }

    /// Run actor initialization
    pub async fn initialize(&mut self) -> Result<(), FluxionError<A::Error>> {
        self.actor.initialize(&mut self.context).await
    }

    /// Run actor deinitialization
    pub async fn deinitialize(&mut self) -> Result<(), FluxionError<A::Error>> {
        self.actor.deinitialize(&mut self.context).await
    }

    /// Run actor cleanup
    pub async fn cleanup(&mut self, error: Option<FluxionError<A::Error>>) -> Result<(), FluxionError<A::Error>> {
        self.actor.cleanup(error).await
    }
}

