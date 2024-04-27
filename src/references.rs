//! # References
//! [`ActorRef`]s, or Actor References, are the primary method through which actors control each other.



use crate::{Actor, ActorWrapper, Handler, Message};
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
pub trait MessageSender<M: Message> {
    /// Sends the given message and waits for a response
    async fn send(&self, message: M) -> M::Result;
}

#[repr(transparent)]
pub struct LocalRef<A: Actor>(pub(crate) slacktor::ActorHandle<ActorWrapper<A>>);



#[async_trait::async_trait]
impl<A: Handler<M>, M: Message> MessageSender<M> for LocalRef<A> {
    async fn send(&self, message: M) -> M::Result {
        self.0.send(message).await
    }
}
