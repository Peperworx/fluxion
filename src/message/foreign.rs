//! # Foreign
//! This module contains structs used by foreign messages.


use core::marker::PhantomData;

use crate::{ActorId, types::serialize::MessageSerializer, MessageSender, Message, SendError};

use alloc::vec::Vec;

#[cfg(async_trait)]
use alloc::boxed::Box;
use serde::{Serialize, Deserialize};



/// # [`ForeignMessage`]
/// Contains the wrapped message, alongside a responder and associated metadata.
pub struct ForeignMessage {
    /// The message's target; the actor that the message is being sent to.
    pub target: ActorId,
    /// The message's source; the actor that sent the message.
    pub source: ActorId,
    /// The contents of the message
    pub message: Vec<u8>,
    /// The channel to respond to the message over.
    /// If None is received, the foreign actor does not exist.
    pub responder: async_oneshot::Sender<Option<Vec<u8>>>,
}


/// # [`ForeignHandle`]
/// A handle for foreign messages that wraps the outbound foreign message channel
pub struct ForeignHandle<S: MessageSerializer> {
    /// The outbound foreign message channel
    channel: whisk::Channel<ForeignMessage>,
    /// The id of the target actor
    target: ActorId,
    /// The owner of this handle
    owner: ActorId,
    /// Phantom data to force a single serializer.
    _phantom: PhantomData<S>,
}

impl<S: MessageSerializer> ForeignHandle<S> {
    #[must_use]
    pub fn new(channel: whisk::Channel<ForeignMessage>, target: ActorId, owner: ActorId) -> Self {
        Self {
            channel, target,
            owner,
            _phantom: PhantomData
        }
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<S: MessageSerializer, M: Message<Response = R> + Serialize, R: for<'a> Deserialize<'a>> MessageSender<M> for ForeignHandle<S> {
    /// Serializes, sends, and awaits a response for a message targeted at a foreign actor.
    /// See [`MessageSender::request`] for more info.
    async fn request(&self, message: M) -> Result<M::Response, crate::types::errors::SendError> {

        // Serialize the message
        let Some(message) = S::serialize(message) else {
            // Return a send error if this failed
            return Err(SendError::SerializationFailed);
        };

        // Create a responder
        let (responder, waiter) = async_oneshot::oneshot();

        // Create the foreign message
        let message = ForeignMessage {
            target: self.target.clone(),
            source: self.owner.clone(),
            message,
            responder
        };

        // Send the foreign message
        self.channel.send(message).await;

        // Wait for a response and deserialize
        let res = waiter.await.or(Err(SendError::NoResponse))?
            .ok_or(SendError::NoResponse)?;

        let Some(res) = S::deserialize::<R>(res) else {
            return Err(SendError::NoResponse);
        };

        Ok(res)
    }
}