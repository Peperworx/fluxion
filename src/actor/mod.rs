//! Contains the implementation of actors and surrounding types.

use crate::error::ActorError;

use self::context::ActorContext;

/// Contains the context that is passed to the actor which allows it to interact with the system
pub mod context;


/// # Actor
/// The core [`Actor`] trait must be implemented for every actor.
/// This trait requires that any implementor be [`Send`] + [`Sync`] + `'static`.
/// This trait uses the `async_trait` crate to allow async functions to be contained,
/// but this will be replaced as soon as async functions in traits is stabilized.
#[async_trait::async_trait]
pub trait Actor: Send + Sync + 'static {

    /// Called upon actor initialization, when the supervisor begins to run.
    async fn initialize(&mut self, context: &mut ActorContext) -> Result<(), ActorError>;

    /// Called upon actor deinitialization, when the supervisor stops.
    /// Note that this will not be called if the initialize function fails.
    /// For handling cases of initialization failure, use [`Actor::cleanup`]
    async fn deinitialize(&mut self, context: &mut ActorContext) -> Result<(), ActorError>;

    /// Called when the actor supervisor is killed, either as the result of a graceful shutdown
    /// or if initialization fails.
    async fn cleanup(&mut self) -> Result<(), ActorError>;
}