//! The [`System`] (primarily implemented by [`super::Fluxion`]) manages the lifecycles of actors.
//! 
//! The core functions are provided in the trait [`System`] to reduce code duplication for implementors of [`super::Context`].

#[cfg(foreign)]
use serde::{Serialize, Deserialize};

use crate::{FluxionParams, Actor, actor::handle::LocalHandle, Handler, Message, ActorId, MessageSender};
use alloc::boxed::Box;

/// # Implementors of [`System`] are responsible for the management of actors. See [`super::Fluxion`].
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait System<C: FluxionParams> {

    /// Add an actor to the system
    /// 
    /// # Returns
    /// Returns [`None`] if the actor was not added to the system.
    /// If the actor was added to the system, returns [`Some`]
    /// containing the actor's [`LocalHandle`].
    async fn add<A: Actor<C>>(&self, actor: A, id: &str) -> Option<LocalHandle<C, A>>;

    /// Initializes foreign message support for an actor.
    /// 
    /// `actor_id` should be the id of the local actor, while `foreign_id` should be how foreign actors
    /// identify this actor. Only one message type per `foreign_id` is supported.
    /// 
    /// # Returns
    /// Returns `true` if the foreign proxy is successfully setup, and `false` if the `foreign_id` is already used.
    #[cfg(foreign)]
    async fn foreign_proxy<A, M, R>(&self, actor_id: &str, foreign_id: &str) -> bool
    where
        A: Handler<C, M>,
        M: Message<Response = R> + Serialize + for<'a> Deserialize<'a>,
        R: Send + Sync + 'static + Serialize + for<'a> Deserialize<'a>;

    /// Get a local actor as a `LocalHandle`. Useful for running management functions like shutdown
    /// on known local actors.
    async fn get_local<A: Actor<C>>(&self, id: &str) -> Option<LocalHandle<C, A>>;

    /// Get an actor from its id as a `Box<dyn MessageSender>`.
    /// Use this for most cases, as it will also handle foreign actors.
    async fn get<
        A: Handler<C, M>,
        #[cfg(not(foreign))] M: Message,
        #[cfg(foreign)] M: Message<Response = R> + Serialize,
        #[cfg(foreign)] R: for<'a> Deserialize<'a>>(&self, id: ActorId) -> Option<Box<dyn MessageSender<M>>>;

    /// Removes an actor from the system, and waits for it to stop execution
    async fn remove(&self, id: &str);
}