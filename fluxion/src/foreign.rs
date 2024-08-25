//! # Foreign Messages
//! This module provides traits and utilities for implementing foreign message handlers.

#[cfg(feature="foreign")]
use alloc::sync::Arc;

#[cfg(feature="foreign")]
use crate::{Handler, Identifier, MessageSender, IndeterminateMessage};



/// # [`Delegate`]
/// A [`Delegate`] is a struct that serves as an interface between one Fluxion instance and every other instance.
/// A [`Delegate`]'s role is simply to provide the Fluxion instance with an implementor of [`ActorRef`] for a given actor ID, and nothing more.
/// This implementation of [`ActorRef`] may wrap a channel, network connection, or simply another [`ActorRef`].
/// All that matters is that this [`ActorRef`] refers to a foreign actor on the given system with the given id.
/// The [`Delegate`] should return [`None`] if no actor with the given ID can be found or is local.
pub trait Delegate: Send + Sync + 'static {
    /// # [`Delegate::get_actor`]
    /// Retrieves an [`ActorRef`] for the given foreign actor.
    #[cfg(all(feature="foreign", not(feature="serde")))]
    fn get_actor<A: Handler<M>, M: IndeterminateMessage>(&self, id: Identifier) -> impl core::future::Future<Output = Option<Arc<dyn MessageSender<M>>>> + Send;

    /// # [`Delegate::get_actor`]
    /// Retrieves an [`ActorRef`] for the given foreign actor.
    #[cfg(all(feature="foreign", feature="serde"))]
    fn get_actor<A: Handler<M>, M: IndeterminateMessage>(&self, id: Identifier) -> impl core::future::Future<Output = Option<Arc<dyn MessageSender<M>>>> + Send
        where M::Result: serde::Serialize + for<'a> serde::Deserialize<'a>;
}

// Delegate is implemented for () as a no-op
impl Delegate for () {
    #[cfg(all(feature="foreign", not(feature="serde")))]
    async fn get_actor<A: Handler<M>, M: IndeterminateMessage>(&self, id: Identifier<'_>) -> Option<Arc<dyn MessageSender<M>>> {
        let _ = id;
        None
    }


    #[cfg(all(feature="foreign", feature="serde"))]
    async fn get_actor<A: Handler<M>, M: IndeterminateMessage>(&self, id: Identifier<'_>) -> Option<Arc<dyn MessageSender<M>>>
        where M::Result: serde::Serialize + for<'a> serde::Deserialize<'a> {
            let _ = id;
            None
    }
}

