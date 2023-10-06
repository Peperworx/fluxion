//! # Types
//! This module contains many of the types and traits used throughout Fluxion.



pub mod errors;

pub mod actor;

pub mod message;

pub mod params;

pub mod executor;

pub mod context;


#[cfg(notification)]
pub mod broadcast;

use self::{errors::ActorError, actor::{Actor, ActorContext}, message::Message};


// Needed by async_trait.
#[cfg(async_trait)]
use alloc::boxed::Box;



/// # Handle
/// Actors MAY implement this trait to handle messages or notifications.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait Handle<M: Message>: Actor {
    async fn message(
        &self,
        message: &M,
        _context: &ActorContext<Self::Params>,
    ) -> Result<M::Response, ActorError<Self::Error>>;
}