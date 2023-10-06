//! Contains structures that allow an actor to access its outside world.

use crate::{handle::ActorHandle, system::System, ActorMap};

use super::{executor::Executor, params::FluxionParams};

use alloc::{boxed::Box, sync::Arc};
use maitake_sync::RwLock;

/// # [`ContextBackend`]
/// An internal trait implemented on a System that abstracts away the system's generics so that it can be used in an actor context.
/// 
/// This also requires the system to implement [`Executor`], which should spawn tasks using the executor provided to the system
/// through its parameters.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub(crate) trait ContextBackend: Executor {

    /// Retrieve an Arc clone of the actor map
    fn get_actors(&self) -> Arc<RwLock<ActorMap>>;
}

