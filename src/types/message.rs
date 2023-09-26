//! # Message Tyes
//! Contains types and traits related to messages.

// Needed by async_trait.
#[cfg(async_trait)]
use alloc::boxed::Box;

#[cfg(serde)]
use serde::{Deserialize, Serialize};

use super::{actor::Actor, Handle, errors::ActorError};


/// # Message
/// This trait is used to mark Messages. Notifications are just Messages with a response type of `()`.
/// By default, all Messages and their responses must be [`Send`] + [`Sync`] + [`'static`].
#[cfg(not(serde))]
pub trait Message: Send + Sync + 'static {
    /// The message's response
    type Response: Send + Sync + 'static;
}
#[cfg(serde)]
pub trait Message: Serialize + for<'a> Deserialize<'a> + Send + Sync + 'static {
    /// The message's response
    type Response: Serialize + for<'a> Deserialize<'a> + Send + Sync + 'static;
}


/// # Handled
/// Not to be confused with [`Handle`], this is a trait implemented for every message for which an [`Actor`] implements [`Handle`].
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait Handled<A: Actor>: Message {
    async fn handle(&self, actor: &A) -> Result<Self::Response, ActorError<A::Error>>;
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<T, A> Handled<A> for T
where
    T: Message,
    A: Actor + Handle<T> {

    async fn handle(&self, actor: &A) -> Result<Self::Response, ActorError<<A as Actor>::Error>> {
        actor.message(self).await
    }
}