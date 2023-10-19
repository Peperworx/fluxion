//! Provides an actor "handle", which enables communication with an actor.

use core::any::Any;

use crate::{FluxionParams, Actor, InvertedHandler, Message, SendError, Handler, InvertedMessage, MessageSender};

use alloc::boxed::Box;

/// # [`ActorHandle`]
/// A trait used when storing an actor handle in the system.
pub(crate) trait ActorHandle: Send + Sync + 'static {
    /// Returns this stored actor as an any type, which allows us to downcast it
    /// to a concrete type.
    fn as_any(&self) -> &dyn Any;
}




/// # [`LocalHandle`]
/// This struct wraps an mpsc channel which communicates with an actor running on the local system.
pub struct LocalHandle<C: FluxionParams, A: Actor<C>> {
    /// The channel that we wrap.
    pub(crate) sender: whisk::Channel<Option<Box<dyn InvertedHandler<C, A>>>>,
}


// Weird clone impl so that Actors do not have to implement Clone.
impl<C: FluxionParams, A: Actor<C>> Clone for LocalHandle<C, A> {
    fn clone(&self) -> Self {
        Self { sender: self.sender.clone() }
    }
}


impl<C: FluxionParams, A: Actor<C>> LocalHandle<C, A> {

    /// Sends a message to the actor and waits for a response
    /// 
    /// # Errors
    /// Returns an error if no response is received
    pub async fn request<M: Message>(&self, message: M) -> Result<M::Response, SendError>
    where
        A: Handler<C, M> {

        // Create the message handle
        let (mh, rx) = InvertedMessage::new(message);

        // Send the handler
        self.sender.send(Some(Box::new(mh))).await;

        // Wait for a response
        rx.await.or(Err(SendError::NoResponse))
    }
}


/// [`MessageSender<M>`] is implemented on [`LocalHandle<A>`] for every message for which `A`
/// implements [`Handler`]
#[cfg_attr(async_trait, async_trait::async_trait)]
impl<C: FluxionParams, A: Handler<C, M>, M: Message> MessageSender<M> for LocalHandle<C, A> {
    async fn request(&self, message: M) -> Result<<M>::Response, SendError> {
        self.request(message).await
    }
}


impl<C: FluxionParams, A: Actor<C>> ActorHandle for LocalHandle<C, A> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}