//! Contains structures that enable an actor to interface with the system.


#[cfg(foreign)]
use serde::{Serialize, Deserialize};

use alloc::boxed::Box;

use crate::{ActorId, System, FluxionParams, Fluxion, Actor, MessageSender, Handler, Message};

use super::handle::LocalHandle;




/// # [`ActorContext`]
/// Provides information about an actor, as well as an interface to the system.
/// 
/// [`ActorContext`] implements the [`System`] trait, which means that it can be used anywhere that
/// a [`Fluxion`] instance can. It should be noted that any message sent by actor handles returned by this struct
/// will be detected by the receiving actor as being sent by this actor. This may cause undesired behavior,
/// so as a rule of thumb, actor handles should not escape the actor that created them.
/// 
/// # Generics
/// This struct takes a single generic which contains configuration data used by the [`Fluxion`] system,
/// as well as other parts of fluxion.
pub struct ActorContext<C: FluxionParams> {
    /// The actor's ID
    id: ActorId,
    /// The system wrapped by this context.
    system: Fluxion<C>
}

impl<C: FluxionParams> ActorContext<C> {
    /// Creates a new [`ActorContext`] from an [`ActorId`] and [`Fluxion`] instance.
    #[must_use]
    pub fn new(id: ActorId, system: Fluxion<C>) -> Self {
        Self {
            id,
            system
        }
    }

    /// Retrieves the actor's id
    #[must_use]
    pub fn get_id(&self) -> ActorId {
        self.id.clone()
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<C: FluxionParams> System<C> for ActorContext<C> {
    /// Add an actor to the system
    /// 
    /// Delegates to the [`Fluxion`] instance's internal add function, specifying the actor
    /// that owns the context as the owner of the returned handle.
    /// See [`System::add`] for more info.
    async fn add<A: Actor<C>>(&self, actor: A, id: &str) -> Option<LocalHandle<C, A>> {
        // Use this actor as the owner
        self.system.add_internal(actor, id, Some(self.id.clone())).await
    }

    /// Delegated directly to [`System::foreign_proxy`], implemented on [`Fluxion`].
    /// See [`System::foreign_proxy`] for more info.
    #[cfg(foreign)]
    async fn foreign_proxy<A, M, R>(&self, actor_id: &str, foreign_id: &str) -> bool
    where
        A: Handler<C, M>,
        M: Message<Response = R> + Serialize + for<'a> Deserialize<'a>,
        R: Send + Sync + 'static + Serialize + for<'a> Deserialize<'a>,
    {
        // Relay to the system's implementation
        self.system.foreign_proxy::<A, M, R>(actor_id, foreign_id).await
    }

    

    /// Get a local actor as a [`LocalHandle`]. Useful for running management functions like shutdown
    /// on known local actors.
    /// 
    /// Delegates to the [`Fluxion`] instance's internal get_local function, specifying the actor
    /// that owns the context as the owner of the returned handle.
    /// See [`System::get_local`] for more info.
    async fn get_local<A: Actor<C>>(&self, id: &str) -> Option<LocalHandle<C, A>> {
        // Use this actor as the owner
        self.system.get_local_internal(id, Some(self.id.clone())).await
    }

    /// Get an actor from its id as a `Box<dyn MessageSender>`.
    /// Use this for most cases, as it will also handle foreign actors.
    /// 
    /// Delegates to the [`Fluxion`] instance's internal get function, specifying the actor
    /// that owns the context as the owner of the returned handle.
    /// See [`System::get`] for more info.
    #[cfg(foreign)]
    async fn get<
        A: Handler<C, M>,
        M: Message + Serialize>(&self, id: ActorId) -> Option<Box<dyn MessageSender<M>>>
    where
        M::Response: for<'a> Deserialize<'a>
    {
        self.system.get_internal::<A,M>(id, Some(self.id.clone())).await
    }
        
    #[cfg(not(foreign))]
    async fn get<
        A: Handler<C, M>,
        M: Message>(&self, id: ActorId) -> Option<Box<dyn MessageSender<M>>>
    {
        self.system.get_internal::<A,M>(id, Some(self.id.clone())).await
    }
    

    /// Removes an actor from the system, and waits for it to stop execution
    /// Delegated directly to [`System::remove`], implemented on [`Fluxion`].
    /// See [`System::remove`] for more info.
    async fn remove(&self, id: &str) {
        // Delegate to the system's implementation
        self.system.remove(id).await;
    }
}