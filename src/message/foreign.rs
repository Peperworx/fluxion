use core::marker::PhantomData;

use crate::{ActorId, types::serialize::MessageSerializer, MessageSender, Message, SendError};

use alloc::vec::Vec;

#[cfg(async_trait)]
use alloc::boxed::Box;
use serde::{Serialize, Deserialize};



/// # [`ForeignMessage`]
/// Contains data for foreign messages that are used in their routing and resolution.
pub struct ForeignMessage {
    /// The message's target; the actor that the message is being sent to.
    pub target: ActorId,
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
    /// Phantom data to force a single serializer.
    _phantom: PhantomData<S>,
}

impl<S: MessageSerializer> ForeignHandle<S> {
    #[must_use]
    pub fn new(channel: whisk::Channel<ForeignMessage>, target: ActorId) -> Self {
        Self {
            channel, target,
            _phantom: PhantomData
        }
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<S: MessageSerializer, M: Message<Response = R> + Serialize, R: for<'a> Deserialize<'a>> MessageSender<M> for ForeignHandle<S> {
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