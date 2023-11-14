//! # Handle
//! 
//! Provides traits and structs for directly interacting with specific actors.

use core::any::Any;
use crate::alloc::string::ToString;

use crate::{FluxionParams, Actor, InvertedHandler, Message, SendError, Handler, InvertedMessage, MessageSender, Event, ActorId};

use alloc::boxed::Box;

use super::ActorControlMessage;

/// # [`ActorHandle`]
/// This trait is implemented for every actor handle, and simply provides methods that are generic to every actor.
/// This trait is then stored directly in the system's internal map.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub(crate) trait ActorHandle: Send + Sync + 'static {
    /// Returns this stored actor as an any type, which allows us to downcast it
    /// to a concrete type.
    fn as_any(&self) -> &dyn Any;

    /// Begins the actor's shutdown process, returning a channel
    /// that will respond when the shutdown is complete.
    async fn begin_shutdown(&self) -> Option<async_oneshot::Receiver<()>>;
}




/// # [`LocalHandle`]
/// Provides an interface with a local actor.
/// This is acheived by wrapping an mpsc channel, the other end of which is held by the actor's supervisor.
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
    /// 
    /// This function creates an inverted message handler and sends it over the channel, waiting for a response.
    /// This function is internal because instead of directly taking a message, it takes an Event, which can have
    /// different contents depending on where this function was called from.
    /// 
    /// # Errors
    /// Returns a [`SendError::NoResponse`] if no response was received from the actor.
    #[cfg_attr(tracing, tracing::instrument(skip(self, message)))]
    pub(crate) async fn request_internal<M: Message>(&self, message: Event<M>) -> Result<M::Response, SendError>
    where
        A: Handler<C, M> {
        
        
        // Create the message handle
        let (mh, rx) = InvertedMessage::new(message);

        // Send the handler
        self.sender.send(ActorControlMessage::Message(Box::new(mh))).await;
        
        if let Some(id) = &self.owner {
            crate::event!(tracing::Level::TRACE, "[{}] Request sent from local handle. Awaiting response.", id);
        } else {
            crate::event!(tracing::Level::TRACE, "[system] Request sent from local handle. Awaiting response.");
        }
        

        // Wait for a response
        let res = rx.await.or(Err(SendError::NoResponse));
        
        if let Some(id) = &self.owner {
            crate::event!(tracing::Level::TRACE, "[{}] Response received.", id);
        } else {
            crate::event!(tracing::Level::TRACE, "[system] Response received.");
        }
        res
    }

    /// Sends a message to the actor and waits for a response
    /// 
    /// # Errors
    /// Returns a [`SendError::NoResponse`] if no response was received from the actor.
    #[cfg_attr(tracing, tracing::instrument(skip(self, message)))]
    pub async fn request<M: Message>(&self, message: M) -> Result<M::Response, SendError>
    where
        A: Handler<C, M> {
        
        if let Some(id) = &self.owner {
            crate::event!(tracing::Level::TRACE, "[{}] Sending request from local handle.", id);
        } else {
            crate::event!(tracing::Level::TRACE, "[system] Sending request from local handle.");
        }
        let event = Event {
            message,
            source: self.owner.clone(),
            target: self.target.clone(),
        };

        self.request_internal(event).await
    }


    /// Shutdown the actor, and wait for the actor to either acknowledge shutdown or drop the channel.
    #[cfg_attr(tracing, tracing::instrument(skip(self)))]
    pub async fn shutdown(&self) {
        if let Some(id) = &self.owner {
            crate::event!(tracing::Level::DEBUG, "[{}] Shutting down actor from local handle.", id);
        } else {
            crate::event!(tracing::Level::TRACE, "[system] Shutting down actor from local handle.");
        }

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
    /// See [`MessageSender::request`] and [`LocalHandle::request`] for more info.
    async fn request(&self, message: M) -> Result<<M>::Response, SendError> {
        self.request(message).await
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<C: FluxionParams, A: Actor<C>> ActorHandle for LocalHandle<C, A> {
    /// See [`ActorHandle::as_any`] for more info.
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// See [`ActorHandle::begin_shutdown`] for more info.
    #[cfg_attr(tracing, tracing::instrument(skip(self)))]
    async fn begin_shutdown(&self) -> Option<async_oneshot::Receiver<()>> {
        crate::event!(tracing::Level::DEBUG, actor=match &self.owner {
            Some(a) => a.to_string(),
            None => "None".to_string()
        }, "Began actor shutdown from handle.");

        // Create a channel for the actor to acknowledge the shutdown on
        let (tx, rx) = async_oneshot::oneshot();

        // Send the message
        self.sender.send(ActorControlMessage::Shutdown(tx)).await;

        Some(rx)
    }
}