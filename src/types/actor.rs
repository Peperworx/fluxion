//! # Actor
//! This module contains types and traits relatign to actors.


// Needed by async_trait.
#[cfg(async_trait)]
use alloc::boxed::Box;
use alloc::sync::Arc;

use crate::{handle::LocalHandle, system::System};

use super::{errors::ActorError, Handle, message::{Message, MessageSender}, params::FluxionParams};



/// # [`ActorContext`]
/// This struct allows an actor to interact with other actors
#[derive(Clone)]
pub struct ActorContext<Params: FluxionParams>(pub(crate) System<Params>, pub(crate) ActorId);

impl<Params: FluxionParams> ActorContext<Params> {
    pub async fn add<A: Actor<Params = Params>>(&self, actor: A, id: &str) -> Option<LocalHandle<A>> {
        self.0.add(actor, id).await
    }

    pub async fn get_local<A: Actor>(&self, id: &str) -> Option<LocalHandle<A>> {
        self.0.get_local(id).await
    }

    pub async fn get<A: Handle<M>, M: Message>(&self, id: ActorId) -> Option<Box<dyn MessageSender<M>>> {
        self.0.get::<A,M>(id).await
    }
}

/// # Actor
/// This trait must be implemented for all Actors. It contains three functions, [`Actor::initialize`], [`Actor::deinitialize`], and [`Actor::cleanup`].
/// Each have a default implementation which does nothing.
///
/// ## Params
/// The associated type Params is used to send around generics used by a bunch of internal structures.
/// This should just be "promoted" to a generic.
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

    type Params: FluxionParams;

    /// The function run upon actor initialization
    async fn initialize(
        &mut self,
        _context: &ActorContext<Self::Params>,
    ) -> Result<(), ActorError<Self::Error>> {
        Ok(())
    }

    /// The function run upon actor deinitialization
    async fn deinitialize(
        &mut self,
        _context: &ActorContext<Self::Params>,
    ) -> Result<(), ActorError<Self::Error>> {
        Ok(())
    }

    /// The function run upon actor cleanup
    async fn cleanup(
        &mut self,
        _context: &ActorContext<Self::Params>,
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
