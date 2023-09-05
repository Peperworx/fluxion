//! # `ActorRef`
//! Contains [`ActorRef`], which wraps the communication channel with an actor, exposing a public interface.

use crate::{
    error::FluxionError,
    message::{Message, MessageGenerics, MessageHandler},
};

use super::supervisor::SupervisorMessage;

/// # `ActorRef`
/// The primary clonable method of communication with an actor.
pub struct ActorRef<G: MessageGenerics> {
    /// The message sender for sending messages to the actor
    pub(crate) message_sender: flume::Sender<SupervisorMessage<G>>,
}

impl<M: MessageGenerics> ActorRef<M> {
    /// Send a message to the actor and wait for a response
    ///
    /// # Errors
    /// This function may generate two possible errors:
    /// * [`FluxionError::SendError`] is returned if the underlying mpmc channel fails transmission.
    /// This happens when there are no longer any receivers, meaning that the actor supervisor has been stopped or has crashed.
    /// * [`FluxionError::ResponseFailed`] is returned when the response receiver fails to receive a response.
    /// This only ever happens when the actor drops the response sender, which may be caused by an error in the actor's handling of the message.
    ///
    pub async fn request(
        &self,
        message: M::Message,
    ) -> Result<<M::Message as Message>::Response, FluxionError<<M::Message as Message>::Error>>
    {
        // Create a oneshot for the message
        let (responder, response) = async_oneshot::oneshot();

        // Create the handler
        let handler = SupervisorMessage::Message(MessageHandler::new(message, responder));

        // Send the handler
        self.message_sender
            .send_async(handler)
            .await
            .or(Err(FluxionError::SendError))?;

        // Wait for a response
        let res = response.await.or(Err(FluxionError::ResponseFailed))?;

        Ok(res)
    }
}
