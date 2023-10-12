//! Provides the [`System`] trait, which is implemented for both [`Fluxion`] and the underlying types behind [`Context`],
//! and is used for unifying their shared functionality.

use crate::{types::{actor::{Actor, ActorId}, Handle, message::{Message, MessageSender}, context::Context}, handle::LocalHandle};

use alloc::boxed::Box;



/// # [`System`]
/// This trait provides functionality for managing actors running on a [`Fluxion`] instance.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait System {
    
    /// The type of the context that all actors on the system are backed by.
    type Context: Context;

    /// Adds an actor to the system, and returns a handle
    async fn add<A: Actor<Context = Self::Context>>(&self, actor: A, id: &str) -> Option<LocalHandle<A>>;

    /// Retrieves a [`LocalHandle`], which allows every message type an actor supports to be used.
    /// This will return None if the actor does not exist.
    async fn get_local<A: Actor<Context = Self::Context>>(&self, id: &str) -> Option<LocalHandle<A>>;

    /// Retrieves a [`MessageSender`] for a given actor id and message. This will be a [`LocalHandle`]
    /// if the actor is on the current system, but will be a [`ForeignHandle`] for foreign actors. [`None`] will
    /// be returned if the target is the local system, but the actor is not found. Actors on foreign systems will
    /// always be returned, as long as foreign messages are enabled. If they are not, then None will be returned
    /// for all foreign actors.
    async fn get<A: Handle<M, Context = Self::Context>, M: Message>(&self, id: ActorId) -> Option<Box<dyn MessageSender<M>>>;

    /// Removes a given actor from the internal map. The actor will stop running when all handles referencing it
    /// are dropped.
    async fn remove(&self, id: &str);
}
