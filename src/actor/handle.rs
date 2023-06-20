//! Contains [`ActorHandle`], a struct that is used for interacting with Actors, and other supporting types.

use std::any::Any;

use tokio::sync::{mpsc, oneshot};

use crate::{message::{LocalMessage, Message, foreign::{ForeignMessage, ForeignReciever}, DualMessage}, error::ActorError};

use super::ActorEntry;



/// # ActorHandle
/// [`ActorHandle`] provides a method through which to interact with actors.
#[derive(Clone)]
pub struct ActorHandle<F, M>
where
    F: Message,
    M: Message, {
    
    /// The message sender for the actor
    pub(crate) message_sender: mpsc::Sender<DualMessage<F, M>>,

    /// The id of the actor
    pub(crate) id: String,
}

impl<F, M> ActorHandle<F, M>
where
    F: Message,
    M: Message, {
    
    /// Gets the actor's id
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Sends a raw message to the actor
    async fn send_raw_message(&self, message: LocalMessage<F, M>) -> Result<(), ActorError> {
        self.message_sender.send(DualMessage::LocalMessage(message)).await.or(Err(ActorError::MessageSendError))
    }

    /// Sends a message to the actor
    pub async fn send(&self, message: M) -> Result<(), ActorError> {
        self.send_raw_message(LocalMessage::<F, M>::Message(message, None)).await
    }

    /// Sends a message to the actor and awaits a response
    pub async fn request(&self, message: M) -> Result<M::Response, ActorError> {
        // Create the responder
        let (responder, reciever) = oneshot::channel();

        // Send the message
        self.send_raw_message(LocalMessage::<F, M>::Message(message, Some(responder))).await?;

        // Await a response
        let res = reciever.await;

        // Return the result with the error converted
        res.or(Err(ActorError::MessageResponseFailed))
    }

    /// Sends a federated message to the actor
    pub async fn send_federated(&self, message: F) -> Result<(), ActorError> {
        self.send_raw_message(LocalMessage::<F, M>::Federated(message, None)).await
    }

    /// Sends a federated message to the actor and awaits a response
    pub async fn request_federated(&self, message: F) -> Result<F::Response, ActorError> {
        // Create the responder
        let (responder, reciever) = oneshot::channel();

        // Send the message
        self.send_raw_message(LocalMessage::<F, M>::Federated(message, Some(responder))).await?;

        // Await a response
        let res = reciever.await;

        // Return the result with the error converted
        res.or(Err(ActorError::FederatedResponseFailed))
    }
}


#[async_trait::async_trait]
impl<F, M> ForeignReciever for ActorHandle<F, M>
where
    F: Message,
    M: Message {

    type Federated = F;

    async fn handle_foreign(&self, foreign: ForeignMessage<F>) -> Result<(), ActorError> {
        self.message_sender.send(DualMessage::ForeignMessage(foreign)).await.or(Err(ActorError::ForeignSendFail))
    }
}


impl<F, M> ActorEntry for ActorHandle<F, M>
where
    F: Message,
    M: Message {
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}