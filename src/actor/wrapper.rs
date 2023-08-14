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

    /// Dispatch a serialized message
    #[cfg(serde)]
    pub async fn dispatch_serialized<M, R, S, D>(&mut self, serializer: S, deserializer: D) -> Result<Vec<u8>, FluxionError<A::Error>>
    where
        M: Message<Response = R> + for<'a> Deserialize<'a>,
        R: Serialize,
        S: Serializer<Ok = Vec<u8>>,
        D: for<'a> Deserializer<'a>,
        A: Handle<M>,
    {

        // Deserialize the message
        let message = M::deserialize(deserializer).or(Err(FluxionError::DeserializeError))?;

        // Dispatch it
        let resp = self.dispatch(message).await?;

        // Serialize the response
        let resp: Vec<u8> = resp.serialize(serializer).or(Err(FluxionError::SerializeError))?;

        Ok(resp)
    }
}

