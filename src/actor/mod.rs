//! # Actor
//! Fluxion's core unit is an Actor. Each Actor MUST implement the [`Actor`] trait, and MAY implement traits for handling different message types.
//! If federated messages are enabled, the actor MUST implement a trait to handle federated messages.
//! If notifications are enabled, the actor MAY implement a trait to handle notifications.

use crate::message::Message;

// Use alloc's version of box to enable async traits
use alloc::boxed::Box;


pub mod supervisor;

pub mod wrapper;

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
/// This trait uses [`async_trait`]. Once async functions in traits are stablized, this dependency will be removed.
#[async_trait::async_trait]
pub trait Actor: Send + Sync + 'static {

    /// The error type returned by the actor
    type Error: Send + Sync + 'static;

    /// The function run upon actor initialization
    async fn initialize(&mut self, _context: ActorContext) -> Result<(), Self::Error> {
        Ok(())
    }

    /// The function run upon actor deinitialization
    async fn deinitialize(&mut self, _context: ActorContext) -> Result<(), Self::Error> {
        Ok(())
    }

    /// The function run upon actor cleanup
    async fn cleanup(&mut self, _error: Option<Self::Error>) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// # Handle
/// Actors MAY implement this trait to handle messages or notifications.
#[async_trait::async_trait]
pub trait Handle<M: Message>: Actor {
    async fn message(&mut self, message: M) -> Result<M::Response, <Self as Actor>::Error>;
}

/// # Handled
/// An iternal trait implemented for all [`Message`]s for which an [`Actor`] implements [`Handle`].
#[async_trait::async_trait]
pub trait Handled<A: Actor + Handle<Self>>: Message + Sized {
    async fn handle(self, actor: &mut A) -> Result<Self::Response, A::Error> {
        actor.message(self).await
    }
}

impl<A, M> Handled<A> for M
    where
        M: Message,
        A: Handle<M> {}