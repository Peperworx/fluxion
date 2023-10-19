//! Contains structures that allow an actor to access its outside world.


#[cfg(foreign)]
use serde::{Serialize, Deserialize};

use alloc::boxed::Box;

use crate::{ActorId, System, FluxionParams, Fluxion, Actor, MessageSender, Handler, Message};

use super::handle::LocalHandle;




/// # [`ActorContext`]
/// Implements [`System`], delegating all calls to the underlying [`Fluxion`].
/// 
pub struct ActorContext<C: FluxionParams> {
    /// The actor's ID
    id: ActorId,
    /// The system wrapped by this context.
    system: Fluxion<C>
}

impl<C: FluxionParams> ActorContext<C> {
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

    async fn add<A: Actor<C>>(&self, actor: A, id: &str) -> Option<LocalHandle<C, A>> {
        self.system.add(actor, id).await
    }

    
    #[cfg(foreign)]
    async fn foreign_proxy<A, M, R, S>(&self, actor_id: &str, foreign_id: &str) -> bool
    where
        A: Handler<C, M>,
        M: Message<Response = R> + Serialize + for<'a> Deserialize<'a>,
        R: Send + Sync + 'static + Serialize + for<'a> Deserialize<'a>,
        S: crate::types::serialize::MessageSerializer {
            self.system.foreign_proxy::<A, M, R, S>(actor_id, foreign_id).await
        }
    
    async fn get_local<A: Actor<C>>(&self, id: &str) -> Option<LocalHandle<C, A>> {
        self.system.get_local(id).await
    }

    
    async fn get<A: Handler<C, M>, M: Message>(&self, id: ActorId) -> Option<Box<dyn MessageSender<M>>> {
        self.system.get::<A, M>(id).await
    }

    /// Removes an actor from the system, and waits for it to stop execution
    async fn remove(&self, id: &str) {
        self.system.remove(id).await;
    }
}