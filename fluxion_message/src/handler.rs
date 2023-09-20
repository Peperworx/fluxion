//! Contains [`MessageHandler`], a struct which contaisn both a message and a responder oneshot.

use fluxion_error::MessageError;

use crate::Message;

/// # `MessageHandler`
/// This is the struct that is actually sent over the channel to an actor and stores both a message and its responder.
/// This is primarilly to reduce repetitive code.
pub struct MessageHandler<M: Message> {
    /// The message
    message: M,
    /// The responder
    responder: async_oneshot::Sender<M::Response>,
}

impl<M: Message> MessageHandler<M> {
    /// Create a new [`MessageHandler`], returning both the [`MessageHandler`] itself and the receiving oneshot.
    pub fn new(message: M) -> (Self, async_oneshot::Receiver<M::Response>) {
        // Create the oneshot.
        let (responder, receiver) = async_oneshot::oneshot();

        (Self { message, responder }, receiver)
    }

    /// Respond to the message
    ///
    /// # Errors
    /// This function may return an error due to a closed channel. This error is unrecoverable.
    pub fn respond(&mut self, response: M::Response) -> Result<(), MessageError> {
        self.responder
            .send(response)
            .or(Err(MessageError::ResponseFailed))
    }

    /// Returns the contained message
    pub fn message(&self) -> &M {
        &self.message
    }
}
