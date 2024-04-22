use alloc::sync::Arc;
use maitake_sync::RwLock;
use slacktor::Slacktor;

use crate::{Actor, ActorWrapper};





/// # [`Fluxion`]
/// Contains the core actor management functionality of fluxion
pub struct Fluxion {
    /// The underlying slacktor instance.
    /// This is wrapped in an [`Arc`]` and [`RwLock`]` to allow concurrent access from different tasks.
    /// The [`RwLock`] is used instead of a mutex because it can be assumed that actor references
    /// will be retrieved more often than actors are created.
    slacktor: Arc<RwLock<Slacktor>>,
    /// The identifier of this system as a string
    system_id: Arc<str>,
}

impl Fluxion {
    /// # [`new`]
    /// Creates a new [`Fluxion`] instance with the given system id
    #[must_use]
    pub fn new(id: &str) -> Self {
        Self {
            slacktor: Arc::new(RwLock::new(Slacktor::new())),
            system_id: id.into(),
        }
    }

    /// # [`get_id`]
    /// Gets the system's id
    #[must_use]
    pub fn get_id(&self) -> &str {
        &self.system_id
    }

    /// # [`spawn`]
    /// Spawns an actor on the local instance, returning its id.
    /// <div class = "info">
    /// Locks the underlying RwLock as write.
    /// </div>
    /// 
    /// # Errors
    /// Returns an error if the actor failed to initialize.
    /// On an error, the actor will not be spawned.
    pub async fn spawn<A: Actor>(&self, mut actor: A) -> Result<usize, A::Error> {

        // Run the actor's initialization code
        actor.initialize().await?;

        // Wrap the actor
        let actor = ActorWrapper(actor);

        // Lock the underlying slacktor instance as write and add the actor
        let id = self.slacktor.write().await.spawn(actor);

        // Return the actor's id.
        Ok(id)
    }

    /// # [`kill`]
    /// Given an actor's id, kills the actor
    pub async fn kill<A: Actor>(&self, id: usize) {
        // Lock the underylying slacktor instance as write and kill the actor
        self.slacktor.write().await.kill::<ActorWrapper<A>>(id).await;
    }

}