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
