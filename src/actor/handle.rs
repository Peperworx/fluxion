use tokio::sync::mpsc;

use super::{ActorMetadata, ActorID, ActorMessage, MessageType};



/// # ActorHandle
/// Provides an interface to communicate with an actor.
#[derive(Clone, Debug)]
pub struct ActorHandle<F: ActorMessage> {
    /// The metadata of the referenced actor
    pub(crate) metadata: ActorMetadata,
    /// The mpsc sender for the federated message
    pub(crate) federated_sender: mpsc::Sender<MessageType<F>>,
}

impl<F: ActorMessage> ActorHandle<F> {
    

    /// Retrieves the actor's id
    pub fn get_id(&self) -> ActorID {
        self.metadata.id.clone()
    }

    /// Sends a federated message to the actor and does not wait for the response.
    pub fn send_federated(&self, message: F) {

    }
}