//! Contains a struct that wraps an implementor of [`Actor`]

use super::{ActorContext, Actor, Handle};

/// # ActorWrapper
/// This struct wraps an implementor of [`Actor`], and enables the registration and dispatch of foreign messages.
pub struct ActorWrapper<A> {
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
            context: ActorContext
        }
    }

    
}