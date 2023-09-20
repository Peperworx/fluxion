//! Contains types for serializing, deserializing, and handling foreign messages.
/// # `ForeignMessage`
/// This struct is similar to a message handler, except it contains a `Vec<u8>` instead of a message.

use alloc::vec::Vec;

use fluxion_error::MessageError;

use serde::{Serialize,Deserialize};

use crate::{Message, serializer::MessageSerializer};


/// # [`DecodeMessage`]
/// If federated messages are enabled, this contains two associated type: Message and Federated.
/// If federated messages are disabled, this only contains Message. This is automatically implemented for (Message, Federated) if `federated`
/// is enabled, and for T: Message if not.
pub trait EncodedMessage {
    type Message: Message;

    #[cfg(federated)]
    type Federated: Message;
}

#[cfg(federated)]
impl<A: Message, B: Message> EncodedMessage for (A, B) {
    type Message = A;
    type Federated = B;
}

#[cfg(not(federated))]
impl<T: Message> EncodedMessage for T {
    type Message = T;
}

/// # [`ForeignMessage`]
/// This struct is similar to a message handler, except it contains a `Vec<u8>` of serialized data instead of a message.
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
    pub fn respond(&mut self, response: Vec<u8>) -> Result<(), MessageError> {
        self.responder
            .send(response)
            .or(Err(MessageError::ResponseFailed))
    }


    /// Decode the contents of the message
    ///
    /// # Errors
    /// May return an error when deserialization fails.
    /// 
    /// # Generics
    /// This decode function decodes both federated and regular messages.
    /// Because of this, it must know both types. However, because [`fluxion_types`] depends on this crate,
    /// it can not use the traits from there, so [`EncodedMessage`] is defined
    pub fn decode<M: EncodedMessage, S: MessageSerializer>(
        &self,
    ) -> Result<ForeignType<M>, MessageError>
    where
        M::Message: for<'a> Deserialize<'a>,
    {
        S::deserialize(&self.message)
    }
}




/// # [`ForeignType`]
/// Marks the type of the foreign message's contents.
#[derive(Serialize, Deserialize)]
pub enum ForeignType<M: EncodedMessage> {
    /// A regular message
    Message(M::Message),
    /// A federated message
    #[cfg(federated)]
    Federated(M::Federated),
}
