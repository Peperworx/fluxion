use tokio::sync::mpsc;

use super::{ActorMetadata, ActorID, ActorMessage, MessageType};



/// # ActorHandle
/// Provides an interface to communicate with an actor.
#[derive(Clone, Debug)]
pub struct ActorHandle {
    /// The metadata of the referenced actor
    pub(crate) metadata: ActorMetadata,
}

impl ActorHandle {
    

    /// Retrieves the actor's id
    pub fn get_id(&self) -> ActorID {
        self.metadata.id.clone()
    }
}