//! Contains the implementation of actors and surrounding types.

use std::any::Any;

use crate::{
    error::ActorError,
    message::{Message, Notification},
};

#[cfg(feature = "foreign")]
use crate::message::foreign::ForeignReceiver;

use self::context::ActorContext;

/// Contains the context that is passed to the actor which allows it to interact with the system
pub mod context;

/// Contains implementation of [`crate::actor::path::ActorPath`], which provides utilities for working with actor identifiers.
pub mod path;

/// Contains [`crate::actor::handle::ActorHandle`], a struct that is used for interacting with Actors, and other supporting types.
pub mod handle;

/// Contains [`crate::actor::supervisor::ActorSupervisor`], a struct containing a task that handles an actor's lifecycle.
pub mod supervisor;

/// # Actor
/// The core [`Actor`] trait must be implemented for every actor.
/// This trait requires that any implementor be [`Send`] + [`Sync`] + `'static`,
/// and uses the `async_trait` crate to allow async functions to be contained,
/// but this will be replaced as soon as async functions in traits are stabilized.
#[async_trait::async_trait]
pub trait Actor: Send + Sync + 'static {
    /// Called upon actor initialization, when the supervisor begins to run.
    async fn initialize<F: Message, N: Notification>(
        &mut self,
        context: &mut ActorContext<F, N>,
    ) -> Result<(), ActorError>;

    /// Called upon actor deinitialization, when the supervisor stops.
    /// Note that this will not be called if the initialize function fails.
    /// For handling cases of initialization failure, use [`Actor::cleanup`]
    async fn deinitialize<F: Message, N: Notification>(
        &mut self,
        context: &mut ActorContext<F, N>,
    ) -> Result<(), ActorError>;

    /// Called when the actor supervisor is killed, either as the result of a graceful shutdown
    /// or if initialization fails.
    async fn cleanup(&mut self) -> Result<(), ActorError>;
}

/// # ActorEntry
/// This trait is used for actor entries in the hashmap, and is automatically implemented for any
/// types that meet its bounds
#[cfg(feature = "foreign")] 
pub(crate) trait ActorEntry: Any + ForeignReceiver {
    fn as_any(&self) -> &dyn Any;
}

#[cfg(not(feature = "foreign"))] 
pub(crate) trait ActorEntry: Any{
    fn as_any(&self) -> &dyn Any;
}
