
use tokio::sync::{mpsc, oneshot};

use crate::error::ActorError;
use super::{ActorMetadata, ActorID,ActorMessage};



/// # ActorHandle
/// Provides an interface to communicate with an actor.
#[derive(Clone, Debug)]
pub struct ActorHandle<F: ActorMessage, M: ActorMessage> {
    /// The metadata of the referenced actor
    metadata: ActorMetadata,
    /// The federated message sender
    federated_sender: mpsc::Sender<(F, Option<oneshot::Sender<F::Response>>)>,
    /// The message sender
    message_sender: mpsc::Sender<(M, Option<oneshot::Sender<M::Response>>)>,
}

impl<F: ActorMessage, M: ActorMessage> ActorHandle<F, M> {
    
    pub fn new(metadata: ActorMetadata,
        federated_sender: mpsc::Sender<(F, Option<oneshot::Sender<F::Response>>)>,
        message_sender: mpsc::Sender<(M, Option<oneshot::Sender<M::Response>>)>) -> Self {
        Self {
            metadata,
            federated_sender,
            message_sender
        }
    }

    /// Retrieves the actor's id
    pub fn get_id(&self) -> ActorID {
        self.metadata.id.clone()
    }

    /// Sends a federated message
    pub async fn send_federated(&self, message: F) -> Result<(), ActorError> {
        match self.federated_sender.send((message, None)).await {
            Ok(_) => Ok(()),
            Err(_) => Err(ActorError::FederatedSendError),
        }
    }

    // Sends a federated message and waits for a response
    pub async fn request_federated(&self, message: F) -> Result<F::Response, ActorError> {

        // Create the oneshot channel for the response
        let (tx, rx) = oneshot::channel();

        // Send the message
        let sent =  self.federated_sender.send((message, Some(tx))).await;

        // If it errored, return an error.
        if sent.is_err() {
            return Err(ActorError::FederatedSendError);
        };

        // Await recieve
        let ret = rx.await;

        // If Ok return, if Err, error
        match ret {
            Ok(v) => Ok(v),
            Err(_) => Err(ActorError::FederatedResponseError),
        }
    }

    /// Send a message
    pub async fn send(&self, message: M) -> Result<(), ActorError> {
        match self.message_sender.send((message, None)).await {
            Ok(_) => Ok(()),
            Err(_) => Err(ActorError::FederatedSendError),
        }
    }

    ///// Sends a federated message and waits for a response
    pub async fn request(&self, message: F) -> Result<F::Response, ActorError> {

        // Create the oneshot channel for the response
        let (tx, rx) = oneshot::channel();

        // Send the message
        let sent =  self.federated_sender.send((message, Some(tx))).await;

        // If it errored, return an error.
        if sent.is_err() {
            return Err(ActorError::FederatedSendError);
        };

        // Await recieve
        let ret = rx.await;

        // If Ok return, if Err, error
        match ret {
            Ok(v) => Ok(v),
            Err(_) => Err(ActorError::FederatedResponseError),
        }
    }
}