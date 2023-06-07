use super::ActorMetadata;



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
}