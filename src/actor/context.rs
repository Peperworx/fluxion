//! The context that is passed to the actor which allows it to interact with the system

use crate::{
    message::{Message, Notification},
    system::{System, GetActorReturn},
};



#[cfg(all(feature = "tracing", debug_assertions))]
use tracing::{event, Level};

use super::ActorID;




/// # ActorContext
/// [`ActorContext`] provides methods to allow an actor to interact with its [`System`] and other actors.
/// This is done instead of providing a [`System`] reference directly to disallow actors from calling notifications and calling into themselves, which
/// can cause infinite loops.
pub struct ActorContext<F: Message, N: Notification> {
    /// The actor's id
    pub(crate) id: ActorID,
    /// The system
    pub(crate) system: System<F, N>,
}

impl<F: Message, N: Notification> ActorContext<F, N> {
    /// Retrieves an actor from the system.
    /// Returns [`None`] if the actor does not exist or if an actor tries to retrieve its own handle.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    pub async fn get_actor<M: Message>(&self, id: &str) -> Option<GetActorReturn<F, M>> {

        #[cfg(all(feature = "tracing", debug_assertions))]
        event!(Level::TRACE, actor=self.id.to_string(), "Retrieving a handle to {} from an actor context", id);

        #[cfg(not(feature = "foreign"))]
        let new_id = id.to_string();
        #[cfg(feature = "foreign")]
        let new_id = super::path::ActorPath::new(id)?;

        // If the the id matches the actor's own path, then return None
        if Some(&new_id) == Some(&self.id) {
            #[cfg(all(feature = "tracing", debug_assertions))]
            event!(Level::TRACE, actor=self.id.to_string(), "Actor attempted to retrieve its own handle.");

            None
        } else {
            self.system.get_actor(id).await
        }
    }

    /// Gets this actor's id
    pub fn get_id(&self) -> &ActorID {
        &self.id
    }
}
