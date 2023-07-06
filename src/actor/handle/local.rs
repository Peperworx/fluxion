/// Contains [`LocalHandle`], an implementor of [`ActorHandle`] used for communicating with local actors.

use std::any::Any;

use tokio::sync::{mpsc, oneshot};

use crate::{
    error::ActorError,
    message::{
        LocalMessage, Message, MT,
    },
    actor::ActorEntry, ActorID,
};


#[cfg(feature = "foreign")]
use crate::message::{
    foreign::{ForeignMessage, ForeignReceiver},
    DualMessage
};

use super::ActorHandle;


/// # LocalHandle
/// [`LocalHandle`] acts as an [`ActorHandle`] for local actors.
/// This works by maintaining an [`mpsc::Sender`] that connects to an [`mpsc::Receiver`]
/// owned by the actor's supervisor.
#[derive(Clone)]
pub struct LocalHandle<F, M>
where
    F: Message,
    M: Message,
{
    /// The message sender for the actor
    pub(crate) message_sender: mpsc::Sender<MT<F, M>>,

    /// The id of the actor
    pub(crate) id: ActorID,
}



impl<F, M> LocalHandle<F, M>
where
    F: Message,
    M: Message,
{
    /// Sends a raw message to the actor
    async fn send_raw_message(&self, message: LocalMessage<F, M>) -> Result<(), ActorError> {
        #[cfg(feature = "foreign")]
        let message = DualMessage::LocalMessage(message);

        self.message_sender
            .send(message)
            .await
            .or(Err(ActorError::MessageSendError))
    }
}





#[async_trait::async_trait]
impl<F, M> ActorHandle<F, M> for LocalHandle<F, M>
where
    F: Message,
    M: Message,
{
    /// Gets the referenced actor's id.
    fn get_id(&self) -> &ActorID {
        &self.id
    }

    /// Sends a message to the referenced actor and does not wait for a response.
    async fn send(&self, message: M) -> Result<(), ActorError> {
        self.send_raw_message(LocalMessage::<F, M>::Message(message, None))
            .await
    }

    /// Sends a message to the actor and waits for a response.
    async fn request(&self, message: M) -> Result<M::Response, ActorError> {
        // Create the responder
        let (responder, reciever) = oneshot::channel();

        // Send the message
        self.send_raw_message(LocalMessage::<F, M>::Message(message, Some(responder)))
            .await?;

        // Await a response
        let res = reciever.await;

        // Return the result with the error converted
        res.or(Err(ActorError::MessageResponseFailed))
    }

    /// Sends a federated message to the referenced actor and does not wait for a response.
    async fn send_federated(&self, message: F) -> Result<(), ActorError> {
        self.send_raw_message(LocalMessage::<F, M>::Federated(message, None))
            .await
    }

    /// Sends a federated message to the referenced actor and waits for a response.
    async fn request_federated(&self, message: F) -> Result<F::Response, ActorError> {
        // Create the responder
        let (responder, reciever) = oneshot::channel();

        // Send the message
        self.send_raw_message(LocalMessage::<F, M>::Federated(message, Some(responder)))
            .await?;

        // Await a response
        let res = reciever.await;

        // Return the result with the error converted
        res.or(Err(ActorError::FederatedResponseFailed))
    }
}



/// [`ForeignReceiver`] is implemented for [`LocalHandle`], allowing the handle to recieve foreign messages
/// without knowing the message type while stored in the system.
#[cfg(feature = "foreign")]
#[async_trait::async_trait]
impl<F, M> ForeignReceiver for LocalHandle<F, M>
where
    F: Message,
    M: Message,
{
    type Federated = F;

    async fn handle_foreign(&self, foreign: ForeignMessage<F>) -> Result<(), ActorError> {
        self.message_sender
            .send(DualMessage::ForeignMessage(foreign))
            .await
            .or(Err(ActorError::ForeignSendFail))
    }
}

/// [`ActorEntry`] is implemenented for [`LocalHandle`] to allow it to upcast to [`Any`] while stored in the system.
impl<F, M> ActorEntry for LocalHandle<F, M>
where
    F: Message,
    M: Message,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}