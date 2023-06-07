use super::{ActorMetadata, ActorID};



/// # ActorHandle
/// Provides an interface to communicate with an actor.
#[derive(Clone, Debug)]
pub struct ActorHandle {
    /// The metadata of the referenced actor
    metadata: ActorMetadata,
}

impl ActorHandle {
    pub fn new(metadata: ActorMetadata) -> Self {
        Self {
            metadata,
        }
    }

    /// Retrieves the actor's id
    pub fn get_id(&self) -> ActorID {
        self.metadata.id.clone()
    }
}