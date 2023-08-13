//! # Actor
//! Fluxion's core unit is an Actor. Each Actor MUST implement the [`Actor`] trait, and MAY implement traits for handling different message types.
//! If federated messages are enabled, the actor MUST implement a trait to handle federated messages.
//! If notifications are enabled, the actor MAY implement a trait to handle notifications.

pub struct ActorContext;

/// # Actor
/// This trait must be implemented for all Actors. It contains three functions, [`Actor::initialize`], [`Actor::deinitialize`], and [`Actor::cleanup`].
/// Each have a default implementation which does nothing.
/// 
/// ## Initialization
/// When an Actor is added to a system, a separate management, or "supervisor" task is started which oversees the Actor's lifetime.
/// When this supervisor task is started, [`Actor::initialize`] is immediately called. If successful, the supervisor begins the Actor's
/// main loop. Upon failure, the supervisor immediately skips to the cleanup phase.
/// 
/// ## Deinitialization
/// If the actor's main loop exits, either gracefully or by an error, [`Actor::deinitialize`] is called. Reguardless of if this function
/// fails or not, the actor skips to the cleanup phase.
/// 
/// ## Cleanup
/// After the supervisor task exits, [`Actor::cleanup`] is called. In place of the ActorContext, an `Option<Self::Error>` is provided, containing None
/// if the supervisor exited gracefully, or `Some(error)` if the supervisor task failed with an error. This function is always called.
/// If [`Actor::cleanup`] returns an error, the error is simply logged if tracing is enabled.
/// 
/// ## Async
/// 
/// This trait uses `async_trait`. Once async functions in traits are stablized, this dependency will be removed.
#[async_trait::async_trait]
pub trait Actor {

    /// The error type returned by the actor
    type Error;

    /// The function run upon actor initialization
    fn initialize(&mut self, _context: ActorContext) -> Result<(), Self::Error> {
        Ok(())
    }

    /// The function run upon actor deinitialization
    fn deinitialize(&mut self, _context: ActorContext) -> Result<(), Self::Error> {
        Ok(())
    }

    /// The function run upon actor cleanup
    fn cleanup(&mut self, _error: Option<Self::Error>) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// # Handler
/// Actors MAY implement this trait to handle messages. 
pub trait Handler<M> {

}