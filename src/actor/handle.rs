//! Provides an actor "handle", which enables communication with an actor.

use core::any::Any;

use crate::{FluxionParams, Actor, InvertedHandler, Message, SendError, Handler, InvertedMessage, MessageSender, Event, ActorId};

use alloc::boxed::Box;
use async_oneshot::Receiver;

use super::ActorControlMessage;

/// # [`ActorHandle`]
/// A trait used when storing an actor handle in the system.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub(crate) trait ActorHandle: Send + Sync + 'static {
    /// Returns this stored actor as an any type, which allows us to downcast it
    /// to a concrete type.
    fn as_any(&self) -> &dyn Any;

    /// Begins the actor's shutdown process, returning a channel
    /// that will respond when the shutdown is complete.
    async fn begin_shutdown(&self) -> Option<Receiver<()>>;
}




/// # [`LocalHandle`]
/// This struct wraps an mpsc channel which communicates with an actor running on the local system.
pub struct LocalHandle<C: FluxionParams, A: Actor<C>> {
    /// The channel that we wrap.
    pub(crate) sender: whisk::Channel<ActorControlMessage<Box<dyn InvertedHandler<C, A>>>>,
    /// The owner of this channel.
    /// None if the channel was taken from the system, and if taken from an actor's context
    /// contains the actor's id.
    pub(crate) owner: Option<ActorId>,
    /// The actor that this message targets
    pub(crate) target: ActorId,
}


// Weird clone impl so that Actors do not have to implement Clone.
impl<C: FluxionParams, A: Actor<C>> Clone for LocalHandle<C, A> {
    fn clone(&self) -> Self {
        Self { sender: self.sender.clone(), owner: self.owner.clone(), target: self.target.clone() }
    }
}


impl<C: FluxionParams, A: Actor<C>> LocalHandle<C, A> {

    /// Internal implementation for request.
    pub(crate) async fn request_internal<M: Message>(&self, message: Event<M>) -> Result<M::Response, SendError>
    where
        A: Handler<C, M> {
        
        
        // Create the message handle
        let (mh, rx) = InvertedMessage::new(message);

        // Send the handler
        self.sender.send(ActorControlMessage::Message(Box::new(mh))).await;

        // Wait for a response
        rx.await.or(Err(SendError::NoResponse))
        // Create the Event wrapping the message
        
    }

    /// Sends a message to the actor and waits for a response
    /// 
    /// # Errors
    /// Returns an error if no response is received
    pub async fn request<M: Message>(&self, message: M) -> Result<M::Response, SendError>
    where
        A: Handler<C, M> {
        
        let event = Event {
            message,
            source: self.owner.clone(),
            target: self.target.clone(),
        };

        self.request_internal(event).await
    }


    /// Shutdown the actor
    pub async fn shutdown(&self) {
        // Create a channel for the actor to acknowledge the shutdown on
        let (tx, rx) = async_oneshot::oneshot();

        // Send the message
        self.sender.send(ActorControlMessage::Shutdown(tx)).await;

        // Await a response.
        // If the channel is dropped, we take no news as good news
        let _ = rx.await;
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

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<C: FluxionParams, A: Actor<C>> ActorHandle for LocalHandle<C, A> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn begin_shutdown(&self) -> Option<Receiver<()>> {
        // Create a channel for the actor to acknowledge the shutdown on
        let (tx, rx) = async_oneshot::oneshot();

        // Send the message
        self.sender.send(ActorControlMessage::Shutdown(tx)).await;

        Some(rx)
    }
}