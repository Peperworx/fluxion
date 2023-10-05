//! # Actor
//! This module contains types and traits relatign to actors.


// Needed by async_trait.
#[cfg(async_trait)]
use alloc::boxed::Box;
use alloc::sync::Arc;

use crate::handle::LocalHandle;

use super::{errors::ActorError, Handle, message::{Message, MessageSender}};



/// # [`ActorContext`]
/// This trait is implemented on the struct that provides an actor's context.
/// This enables nasty generic parameters to be hidden.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait ActorContext {
    /// Adds an actor to the system, and returns a handle.
    async fn add<A: Actor>(&self, actor: A, id: &str) -> Option<LocalHandle<A>>;
    
    /// Retrieves a [`LocalHandle`], which allows every message type an actor supports to be used.
    /// This will return None if the actor does not exist.
    async fn get_local<A: Actor>(&self, id: &str) -> Option<LocalHandle<A>>;

    /// Retrieves a [`MessageSender`] for a given actor id and message. This will be a [`LocalHandle`]
    /// if the actor is on the current system, but will be a [`ForeignHandle`] for foreign actors. [`None`] will
    /// be returned if the target is the local system, but the actor is not found. Actors on foreign systems will
    /// always be returned, as long as foreign messages are enabled. If they are not, then None will be returned
    /// for all foreign actors.
    async fn get<A: Handle<M>, M: Message>(&self, id: ActorId) -> Option<Box<dyn MessageSender<M>>>;
}


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
/// if the supervisor exited gracefully, or `Some(error)` if the supervisor task failed with an error. This function is always called on actor exit.
/// If [`Actor::cleanup`] returns an error, the error is simply logged if tracing is enabled, and the actor stops.
///
/// ## Async
/// This trait uses [`async_trait`] when on stable. Once async functions in traits are stablized, this dependency will be removed.
/// On nightly, the `nightly` feature may be enabled, which uses `#![feature(async_fn_in_trait)]`
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait Actor: Send + Sync + 'static {
    /// The error type returned by the actor
    type Error: Send + Sync + 'static;

    /// The function run upon actor initialization
    async fn initialize<C: ActorContext>(
        &mut self,
        _context: &C,
    ) -> Result<(), ActorError<Self::Error>> {
        Ok(())
    }

    /// The function run upon actor deinitialization
    async fn deinitialize(
        &mut self
    ) -> Result<(), ActorError<Self::Error>> {
        Ok(())
    }

    /// The function run upon actor cleanup
    async fn cleanup(
        &mut self,
        _error: Option<ActorError<Self::Error>>,
    ) -> Result<(), ActorError<Self::Error>> {
        Ok(())
    }
}


/// # [`ActorId`]
/// An actor's id contains two parts: the system the actor is running on, and the actual name of the actor.
/// 
/// This struct represents and provides operations to act on an actor id.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActorId(Arc<str>);

impl ActorId {
    /// Get the actor refered to by the [`ActorId`]
    #[must_use]
    pub fn get_actor(&self) -> &str {
        // Find the index of the first ':', or get the entire string
        let index = self.0.find(':').map_or(0, |v| v+1);
        &self.0[index..]
    }

    /// Get the system refered to by the [`ActorId`]
    #[must_use]
    pub fn get_system(&self) -> &str {
        // Everything before the first ':'
        let index = self.0.find(':').map_or(0, |v| v);
        &self.0[..index]
    }
}

impl<T> From<T> for ActorId
where
    Arc<str>: From<T>
{
    fn from(value: T) -> Self {
        ActorId(value.into())
    }
}
