//! # References
//! [`ActorRef`]s, or Actor References, are the primary method through which actors control each other.



use crate::{Actor, ActorWrapper, Delegate, Handler, Message};
use alloc::boxed::Box;

/// # [`ActorRef`]
/// This trait provides methods for actors to communicate with and control each other.
pub trait ActorRef<A: Actor> {}

/// # [`MessageSender`]
/// This trait provides the ability to send a specific message type to a specific actor.
/// This trait is only necessary because traits with generic methods are not object safe,
/// and we need a way to be generic over multiple types of [`ActorRef`] at once.
/// Sadly, [`async_trait`] is also required for this trait as async fns in traits are not yet object safe either.
#[async_trait::async_trait]
pub trait MessageSender<M: Message>: Send + Sync + 'static {
    /// Sends the given message and waits for a response
    async fn send(&self, message: M) -> M::Result;
}


pub struct LocalRef<A: Actor, D: Delegate>(pub(crate) slacktor::ActorHandle<ActorWrapper<A, D>>, pub(crate) u64);

impl<A: Actor, D: Delegate> LocalRef<A, D> {
    /// # [`LocalRef::get_id`]
    /// Retrieves the actor's ID
    #[must_use]
    pub fn get_id(&self) -> u64 {
        self.1
    }
}

impl<A: Actor, D: Delegate> Clone for LocalRef<A, D> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

#[async_trait::async_trait]
impl<A: Handler<M>, M: Message, D: Delegate> MessageSender<M> for LocalRef<A, D> {
    #[inline]
    async fn send(&self, message: M) -> M::Result {
        self.0.send(message).await
    }
}
