
use tokio::sync::{mpsc, oneshot};

use crate::{error::ActorError, handle_policy};

use super::{ActorMetadata, ActorID,ActorMessage};



/// # ActorHandle
/// Provides an interface to communicate with an actor.
#[derive(Clone, Debug)]
pub struct ActorHandle<F: ActorMessage> {
    /// The metadata of the referenced actor
    metadata: ActorMetadata,
    /// The message sender
    federated_sender: mpsc::Sender<(F, Option<oneshot::Sender<F::Response>>)>
}

impl<F: ActorMessage> ActorHandle<F> {
    
    pub fn new(metadata: ActorMetadata, federated_sender: mpsc::Sender<(F, Option<oneshot::Sender<F::Response>>)>) -> Self {
        Self {
            metadata,
            federated_sender
        }
    }

    /// Retrieves the actor's id
    pub fn get_id(&self) -> ActorID {
        self.metadata.id.clone()
    }

    /// Sends a federated message
    pub async fn send_federated(&self, message: F) -> Result<(), (ActorError, F)> {
        match self.federated_sender.send((message, None)).await {
            Ok(_) => Ok(()),
            Err(e) => Err((ActorError::FederatedSendError, e.0.0)),
        }
    }

    
}