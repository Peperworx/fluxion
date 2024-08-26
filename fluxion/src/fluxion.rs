use alloc::sync::Arc;
use maitake_sync::RwLock;
use slacktor::Slacktor;

use crate::{Actor, ActorContext, ActorWrapper, Delegate, Handler, Identifier, IndeterminateMessage, LocalRef, MessageSender};





/// # [`Fluxion`]
/// Contains the core actor management functionality of fluxion
pub struct Fluxion<D> {
    /// The underlying slacktor instance.
    /// This is wrapped in an [`Arc`] and [`RwLock`] to allow concurrent access from different tasks.
    /// The [`RwLock`] is used instead of a mutex because it can be assumed that actor references
    /// will be retrieved more often than actors are created.
    slacktor: Arc<RwLock<Slacktor>>,
    /// The identifier of this system as a string
    system_id: Arc<str>,
    /// The foreign delegate of this system
    delegate: Arc<D>,
}

impl<D> Clone for Fluxion<D> {
    fn clone(&self) -> Self {
        Self { slacktor: self.slacktor.clone(), system_id: self.system_id.clone(), delegate: self.delegate.clone() }
    }
}

impl<D: Delegate> Fluxion<D> {
    /// # [`Fluxion::new`]
    /// Creates a new [`Fluxion`] instance with the given system id and delegate
    #[must_use]
    pub fn new(id: &str, delegate: D) -> Self {
        Self {
            slacktor: Arc::new(RwLock::new(Slacktor::new())),
            system_id: id.into(),
            delegate: Arc::new(delegate)
        }
    }

    /// # [`Fluxion::get_delegate`]
    /// Gets a reference to the delegate.
    #[must_use]
    pub fn get_delegate(&self) -> &D {
        &self.delegate
    }

    /// # [`Fluxion::get_id`]
    /// Gets the system's id
    #[must_use]
    pub fn get_id(&self) -> &str {
        &self.system_id
    }

    /// # [`Fluxion::add`]
    /// Adds an actor to the local instance, returning its id.
    /// <div class = "info">
    /// Locks the underlying RwLock as write. This will block "management" functionalities such as adding, removing, and retrieving actors, but
    /// will not block any messages.
    /// </div>
    /// 
    /// # Errors
    /// Returns an error if the actor failed to initialize.
    /// On an error, the actor will not be spawned.
    pub async fn add<A: Actor>(&self, mut actor: A) -> Result<u64, A::Error> {

        // Run the actor's initialization code
        actor.initialize().await?;

        // Lock the underlying slacktor instance as write
        let mut system = self.slacktor.write().await;

        // Wrap the actor
        let actor = ActorWrapper(actor, Arc::new(
            ActorContext {
                system: self.clone(),
                id: system.next_id()
            }
        ));

        // Spawn the actor on the slacktor instance
        let id = system.spawn(actor);

        // Return the actor's id.
        Ok(id as u64)
    }

    /// # [`Fluxion::kill`]
    /// Given an actor's id, kills the actor
    /// 
    /// <div class = "info">
    /// Locks the underlying RwLock as write. This will block "management" functionalities such as adding, removing, and retrieving actors, but
    /// will not block any messages.
    /// </div>
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
        self.slacktor.write().await.kill::<ActorWrapper<A, D>>(id).await;
    }


    /// # [`Fluxion::get_local`]
    /// Gets an actor that is known to reside on the local system.
    /// This allows messages that are not serializable to still be used even if Fluxion is compiled with foreign message support.
    /// This function also allows retrieving an actor handle that is capable of sending multiple different messages.
    pub async fn get_local<A: Actor>(&self, id: u64) -> Option<LocalRef<A, D>> {
        // If the id refers to a local actor, lock the slacktor
        // instance as read, and retrieve the handle.
        // The handle is then cloned and returned
        self.slacktor.read().await.get::<ActorWrapper<A, D>>(
            id.try_into().ok()? // If overflow, then the actor does not exist.
        ).cloned()
        .map(|handle| LocalRef(handle, id))
    }

    /// # [`Fluxion::get`]
    /// Retrieves an actor reference capable of communicating using the given message via the given ID.
    #[cfg(feature = "serde")]
    pub async fn get<'a, A: Handler<M>, M: IndeterminateMessage>(&self,
            #[cfg(feature="foreign")] id: impl Into<Identifier<'a>>,
            #[cfg(not(feature="foreign"))] id: impl Into<Identifier>
        ) -> Option<Arc<dyn MessageSender<M>>>
        where M::Result: serde::Serialize + for<'d> serde::Deserialize<'d> {

        match id.into() {
            Identifier::Local(id) => {
                // Get the local ref and wrap in an arc
                self.get_local::<A>(id).await
                    .map(|h| Arc::new(h) as Arc<dyn MessageSender<M>>)
            },
            #[cfg(feature = "foreign")]
            Identifier::Foreign(id, system) => {
                // Send the request on to the delegate
                self.delegate.get_actor::<A, M>(Identifier::Foreign(id, system)).await
            }
        }
    }

    /// # [`Fluxion::get`]
    /// Retrieves an actor reference capable of communicating using the given message via the given ID.
    #[cfg(not(feature = "serde"))]
    pub async fn get<'a, A: Handler<M>, M: IndeterminateMessage>(&self,
            #[cfg(feature="foreign")] id: impl Into<Identifier<'a>>,
            #[cfg(not(feature="foreign"))] id: impl Into<Identifier>
        ) -> Option<Arc<dyn MessageSender<M>>> {

        match id.into() {
            Identifier::Local(id) => {
                // Get the local ref and wrap in an arc
                self.get_local::<A>(id).await
                    .map(|h| Arc::new(h) as Arc<dyn MessageSender<M>>)
            },
            #[cfg(feature = "foreign")]
            Identifier::Foreign(id, system) => {
                // Send the request on to the delegate
                self.delegate.get_actor::<A, M>(Identifier::Foreign(id, system)).await
            }
        }
    }

    /// # [`Fluxion::shutdown`]
    /// Removes all actors from the system and deallocates the underlying slab.
    /// 
    /// <div class = "info">
    /// Locks the underlying RwLock as write. This will block "management" functionalities such as adding, removing, and retrieving actors, but
    /// will not block any messages.
    /// </div>
    pub async fn shutdown(&self) {
        self.slacktor.write().await.shutdown().await;
    }
}
