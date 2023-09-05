//! Contains foreign message types

use alloc::vec::Vec;

use crate::{actor::supervisor::SupervisorMessage, error::FluxionError};

use super::{serializer::MessageSerializer, MessageGenerics};

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
    pub fn new(message: Vec<u8>, responder: async_oneshot::Sender<Vec<u8>>) -> Self {
        Self { message, responder }
    }

    /// Respond to the message
    pub fn respond<E>(&mut self, response: Vec<u8>) -> Result<(), FluxionError<E>> {
        self.responder
            .send(response)
            .or(Err(FluxionError::ResponseFailed))
    }

    /// Decode the contents of the message
    pub fn decode<M: MessageGenerics, S: MessageSerializer, E>(
        &self,
    ) -> Result<ForeignType<M>, FluxionError<E>>
    where
        M::Message: for<'a> Deserialize<'a>,
        M::Federated: for<'a> Deserialize<'a>,
    {
        S::deserialize(&self.message)
    }
}

/// # `ForeignType`
/// Marks the type of the foreign message's contents.
#[derive(Serialize, Deserialize)]
pub enum ForeignType<M: MessageGenerics> {
    /// A regular message
    Message(M::Message),
    /// A federated message
    #[cfg(federated)]
    Federated(M::Federated),
}
