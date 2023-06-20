//! The context that is passed to the actor which allows it to interact with the system

use crate::{system::System, message::{Notification, Message}};

use super::{handle::ActorHandle, path::ActorPath};



/// # ActorContext
pub struct ActorContext<F: Message, N: Notification> {
    /// The actor's path
    pub(crate) path: ActorPath,
    /// The system
    pub(crate) system: System<F, N>,
}

impl<F: Message, N: Notification> ActorContext<F, N> {
    /// Retrieves an actor from the system, returning None if the actor does not exist
    pub async fn get_actor<M: Message>(&self, id: &str) -> Option<Box<dyn ActorHandle<F, M>>> {
        self.system.get_actor(id).await
    }

    /// Gets the actor's path
    pub fn get_path(&self) -> &ActorPath {
        &self.path
    }
}