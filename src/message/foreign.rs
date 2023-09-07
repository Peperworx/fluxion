//! Contains foreign message types

use alloc::vec::Vec;

use crate::{
    error::FluxionError,
    util::generic_abstractions::{ActorParams, MessageParams, SystemParams},
};

use super::serializer::MessageSerializer;

use serde::{Deserialize, Serialize};

/// # `ForeignMessage`
/// This struct is similar to a message handler, except it contains a `Vec<u8>` instead of a message.
pub struct ForeignMessage {
    /// The message
    message: Vec<u8>,
    /// The responder
    responder: async_oneshot::Sender<Vec<u8>>,
}

impl ForeignMessage {
    #[must_use]
    pub fn new(message: Vec<u8>, responder: async_oneshot::Sender<Vec<u8>>) -> Self {
        Self { message, responder }
    }

    /// Respond to the message
    ///
    /// # Errors
    /// This function may return an error due to a closed channel. This error is unrecoverable.
    pub fn respond<E>(&mut self, response: Vec<u8>) -> Result<(), FluxionError<E>> {
        self.responder
            .send(response)
            .or(Err(FluxionError::ResponseFailed))
    }

    /// Decode the contents of the message
    ///
    /// # Errors
    /// May return an error when deserialization fails.
    pub fn decode<AP: ActorParams<S>, S: SystemParams, SD: MessageSerializer, E>(
        &self,
    ) -> Result<ForeignType<AP, S>, FluxionError<E>>
    where
        AP::Message: for<'a> Deserialize<'a>,
        <S::SystemMessages as MessageParams>::Federated: for<'a> Deserialize<'a>,
    {
        SD::deserialize(&self.message)
    }
}

/// # `ForeignType`
/// Marks the type of the foreign message's contents.
#[derive(Serialize, Deserialize)]
pub enum ForeignType<AP: ActorParams<S>, S: SystemParams> {
    /// A regular message
    Message(AP::Message),
    /// A federated message
    #[cfg(federated)]
    Federated(<S::SystemMessages as MessageParams>::Federated),
}
