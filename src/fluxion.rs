use alloc::sync::Arc;
use maitake_sync::RwLock;
use slacktor::Slacktor;

use crate::{Actor, ActorRef, ActorWrapper, Identifier, LocalRef};





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
    pub async fn spawn<A: Actor>(&self, mut actor: A) -> Result<u64, A::Error> {

        // Run the actor's initialization code
        actor.initialize().await?;

        // Wrap the actor
        let actor = ActorWrapper(actor);

        // Lock the underlying slacktor instance as write and add the actor
        let id = self.slacktor.write().await.spawn(actor);

        // Return the actor's id.
        Ok(id as u64)
    }

    /// # [`kill`]
    /// Given an actor's id, kills the actor
    pub async fn kill<A: Actor>(&self, id: u64) {
        // Realistically, it should not be possible for this conversion to ever fail.
        // If the input id is more than usize::MAX, it is most likely an error on the caller's part,
        // as it should be impossible to allocate over usize::MAX actors at all, because
        // each actor has an overhead of more than one byte.
        // We just fail silently here, as it is the same case as the actor not existing.
        let Ok(id) = id.try_into() else {
            return;
        };

        // Lock the underylying slacktor instance as write and kill the actor
        self.slacktor.write().await.kill::<ActorWrapper<A>>(id).await;
    }


    /// # [`get`]
    /// Retrieves an actor reference from the given actor id
    pub async fn get<'a, A: Actor, ID: Into<Identifier<'a>>>(&self, id: ID) -> Option<impl ActorRef> {
        match id.into() {
            Identifier::Local(id) => {

                // If the id refers to a local actor, lock the slacktor
                // instance as read, and retrieve the handle.
                // The handle is then cloned and returned
                let handle = self.slacktor.read().await.get::<ActorWrapper<A>>(
                    id.try_into().ok()? // If overflow, then the actor does not exist.
                ).cloned()?;

                // Wrap the slacktor handle in a LocalReference and return
                Some(LocalRef(handle))
            },
            Identifier::Foreign(_, _) => {
                // Foreign actors not implemented yet.
                // This will involve asking a "ForeignHandler" for an implementor of
                // ActorRef.
                None
            }
        }
    }
}