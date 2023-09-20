//! # `ActorRef`
//! Contains [`ActorRef`], which wraps the communication channel with an actor, exposing a public interface.

use core::any::Any;

use crate::{
    error::MessageError,
    message::{Message, MessageHandler},
    util::generic_abstractions::{ActorParams, SystemParams},
};

#[cfg(foreign)]
use crate::message::foreign::ForeignMessage;

#[cfg(all(async_trait, foreign))]
use alloc::boxed::Box;

use super::{entry::ActorEntry, supervisor::SupervisorMessage};

/// # `ActorRef`
/// The primary clonable method of communication with an actor.
pub struct ActorRef<AP: ActorParams<S>, S: SystemParams> {
    /// The message channel
    pub(crate) messages: flume::Sender<SupervisorMessage<AP, S>>,
    /// The foreign message channel
    #[cfg(foreign)]
    pub(crate) foreign: flume::Sender<ForeignMessage>,
}

// Manual clone impl because of generics.
impl<AP: ActorParams<S>, S: SystemParams> Clone for ActorRef<AP, S> {
    fn clone(&self) -> Self {
        Self {
            messages: self.messages.clone(),
            #[cfg(foreign)]
            foreign: self.foreign.clone(),
        }
    }
}

impl<AP: ActorParams<S>, S: SystemParams> ActorRef<AP, S> {
    /// Send a message to the actor and wait for a response
    ///
    /// # Errors
    /// This function may generate two possible errors:
    /// * [`MessageError::SendError`] is returned if the underlying mpmc channel fails transmission.
    /// This happens when there are no longer any receivers, meaning that the actor supervisor has been stopped or has crashed.
    /// * [`MessageError::ResponseFailed`] is returned when the response receiver fails to receive a response.
    /// This only ever happens when the actor drops the response sender, which may be caused by an error in the actor's handling of the message.
    ///
    pub async fn request(
        &self,
        message: AP::Message,
    ) -> Result<<AP::Message as Message>::Response, MessageError> {
        // Create a oneshot for the message
        let (responder, response) = async_oneshot::oneshot();

        // Create the handler
        let handler = SupervisorMessage::Message(MessageHandler::new(message, responder));

        // Send the handler
        let res = self.messages.send_async(handler).await;
        res.or(Err(MessageError::SendError))?;

        // Wait for a response
        let res = response.await.or(Err(MessageError::ResponseFailed))?;

        Ok(res)
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<AP: ActorParams<S>, S: SystemParams> ActorEntry for ActorRef<AP, S> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(foreign)]
    async fn handle_foreign(&self, message: ForeignMessage) -> Result<(), MessageError> {
        self.foreign
            .send_async(message)
            .await
            .or(Err(MessageError::SendError))
    }
}
