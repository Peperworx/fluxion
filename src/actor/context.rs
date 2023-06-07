use super::{ActorMetadata, ActorID};

pub struct ActorContext {
    pub(crate) metadata: ActorMetadata
}

impl ActorContext {
    /// Gets the actor's id
    pub fn get_id(&self) -> ActorID {
        self.metadata.id.clone()
    }
}