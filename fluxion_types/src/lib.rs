#! [doc = include_str! ("../README.md")]


// The following will be in *every* crate related to fluxion.
#![no_std]
#![warn(clippy::pedantic)]
#![cfg_attr(not(async_trait), feature(async_fn_in_trait))]
#![allow(clippy::module_name_repetitions)]

pub mod atg;
pub mod executor;
pub mod param;


// Needed by async-trait
#[cfg(async_trait)]
extern crate alloc;
#[cfg(async_trait)]
use alloc::boxed::Box;

use fluxion_error::ActorError;
use fluxion_message::Message;

/// # [`ActorContext`]
/// Currently empty.
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
    async fn initialize(
        &mut self,
        _context: &ActorContext,
    ) -> Result<(), ActorError<Self::Error>> {
        Ok(())
    }

    /// The function run upon actor deinitialization
    async fn deinitialize(
        &mut self,
        _context: &ActorContext,
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

/// # Handle
/// Actors may implement this trait to handle messages or notifications.
/// Because the message handler may be spawned in a separate task, message handlers do not have mutable access to 
/// the actor.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait Handle<M: Message>: Actor {
    async fn message(
        &self,
        message: &M,
        _context: &ActorContext,
    ) -> Result<M::Response, ActorError<Self::Error>>;
}
