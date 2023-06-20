//! The context that is passed to the actor which allows it to interact with the system

use crate::{system::System, message::{Notification, Message}};

use super::handle::ActorHandle;



/// # ActorContext
pub struct ActorContext<F: Message, N: Notification> {
    /// The ID of the actor
    pub(crate) id: String,
    /// The system
    pub(crate) system: System<F, N>,
}

impl<F: Message, N: Notification> ActorContext<F, N> {
    /// Retrieves an actor from the system, returning None if the actor does not exist
    pub async fn get_actor<M: Message>(&self, id: &str) -> Option<ActorHandle<F, M>> {
        self.system.get_actor(id).await
    }

    /// Gets the actor's id
    pub fn get_id(&self) -> &str {
        &self.id
    }
}