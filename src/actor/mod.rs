//! # Actor
//! This module contains types and traits relatign to actors.


pub mod context;

pub mod handle;

pub mod supervisor;

use core::fmt::Display;

#[cfg(error_policy)]
use crate::error_policy::ErrorPolicy;

// Needed by async_trait.
#[cfg(async_trait)]
use alloc::boxed::Box;
use alloc::sync::Arc;

use crate::{ActorError, FluxionParams, ActorContext};



/// # [`ActorControlMessage`]
/// Represents different message types used to control a local actor.
pub(crate) enum ActorControlMessage<M> {
    /// A message, which should be dispatched to the actor
    Message(M),
    /// A shutdown command, which should be obeyed, and after the shutdown is complete,
    /// a response should be sent over the oneshot.
    Shutdown(async_oneshot::Sender<()>),
}


/// # Actor
/// This trait must be implemented for all Actors. It contains three functions, [`Actor::initialize`], [`Actor::deinitialize`], and [`Actor::cleanup`].
/// Each have a default implementation which does nothing.
///
/// ## Associated Types
/// The [`Actor`] trait takesan associated type `Error`, which is the custom error type returned by the actor.
///
/// ## Generics
/// The single generic taken by [`Actor`] is configuration data used by Fluxion for compile time config.
/// It is a type implementing the trait [`crate::types::params::FluxionParams`], where each associated type
/// is configurable. This mainly contains information such as what the notification type used by the system is,
/// which async executor is being used, and other related information. Implementors of [`Actor`] may wish
/// to confine themselves to specific parameters (such as a specific executor), and as such this is provided
/// as a generic instead of an associated type.
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
pub trait Actor<C: FluxionParams>: Send + Sync + 'static {
    /// The error type returned by the actor
    type Error: core::fmt::Debug + PartialEq + Send + Sync + 'static;

    /// The error policy to use
    #[cfg(error_policy)]
    const ERROR_POLICY: ErrorPolicy<ActorError<Self::Error>> = ErrorPolicy::default_policy();
    

    /// The function run upon actor initialization
    async fn initialize(
        &mut self,
        _context: &ActorContext<C>
    ) -> Result<(), ActorError<Self::Error>> {
        Ok(())
    }

    /// The function run upon actor deinitialization
    async fn deinitialize(
        &mut self,
        _context: &ActorContext<C>
    ) -> Result<(), ActorError<Self::Error>> {
        Ok(())
    }

    /// The function run upon actor cleanup
    async fn cleanup(
        &mut self,
        _context: &ActorContext<C>,
        _error: Option<&ActorError<Self::Error>>,
    ) -> Result<(), ActorError<Self::Error>> {
        Ok(())
    }
}






/// # [`ActorId`]
/// An actor's id contains two parts: the system the actor is running on, and the actual name of the actor.
/// This struct allows both parts to be retrieved separately from a wrapped [`Arc<str>`].
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

impl AsRef<str> for ActorId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for ActorId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

