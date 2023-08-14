//! Contains a struct that wraps an implementor of [`Actor`]

use core::any::{TypeId, Any};

// Use alloc's version of box to enable async traits
use alloc::{vec::Vec, collections::BTreeMap};

#[cfg(serde)]
use serde::{Serialize, Deserialize, Serializer, Deserializer};


use crate::message::Message;
use crate::error::FluxionError;


use super::{ActorContext, Actor, Handle};

/// # ActorWrapper
/// This struct wraps an implementor of [`Actor`], and enables the registration and dispatch of foreign messages.
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
    pub async fn dispatch<M: Message>(&mut self, message: M) -> Result<M::Response, FluxionError<A::Error>> where A: Handle<M> {
        self.actor.message(message).await
    }
}

