/// Contains [`ForeignHandle`], an implementor of [`ActorHandle`] used for communicating with foreign actors.


use tokio::sync::{mpsc, oneshot};

use crate::{
    error::ActorError,
    message::{
        foreign::{ForeignMessage, ForeignMessenger},
        Message, Notification,
    },
    system::System, actor::path::ActorPath, ActorID,
};

use super::ActorHandle;

#[cfg(all(feature = "tracing", debug_assertions))]
use tracing::{event, Level};

/// # ForeignHandle
/// [`ForeignHandle`] serves as an [`ActorHandle`] for foreign actors.
/// This works by holding an [`mpsc::Sender`] which references the reciever held by the [`System`].
/// When the [`System`] confirms that the corresponding [`mpsc::Receiver`] has been claimed,
/// all messages will be converted to [`ForeignMessage`]s and sent over the channel to be handled by an
/// external task.
pub struct ForeignHandle<F: Message, N: Notification> {
    /// The channel used for sending foreign messages.
    pub(crate) foreign: mpsc::Sender<ForeignMessage<F>>,
    /// The system that holds the reciever for the foreign channel.
    pub(crate) system: System<F, N>,
    /// The path of the foreign actor.
    pub(crate) path: ActorPath,
}

#[async_trait::async_trait]
impl<F: Message, N: Notification> ForeignMessenger for ForeignHandle<F, N> {
    /// The type of the federated message that is sent by both the local and foreign system
    type Federated = F;

    /// This function must be implemented by every [`ForeignMessenger`]. It sends the passed foreign
    /// message to the foreign actor.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self, message)))]
    async fn send_raw_foreign(
        &self,
        message: ForeignMessage<Self::Federated>,
    ) -> Result<(), ActorError> {

        #[cfg(all(feature = "tracing", debug_assertions))]
        event!(Level::TRACE, actor=self.path.to_string(), "Sending a raw foreign message via a ForeignHandle");

        self.foreign
            .send(message)
            .await
            .or(Err(ActorError::ForeignSendFail))
    }

    /// This function must be implemented by every [`ForeignMessenger`]
    /// It must return true if an external task is ready to recieve a foreign message
    async fn can_send_foreign(&self) -> bool {
        self.system.can_send_foreign().await
    }
}

#[async_trait::async_trait]
impl<F: Message, M: Message, N: Notification> ActorHandle<F, M> for ForeignHandle<F, N> {
    /// Gets the referenced actor's id.
    fn get_id(&self) -> &ActorID {
        &self.path
    }

    /// Sends a message to the referenced actor and does not wait for a response.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self, message)))]
    async fn send(&self, message: M) -> Result<(), ActorError> {
        #[cfg(all(feature = "tracing", debug_assertions))]
        event!(Level::TRACE, actor=self.path.to_string(), "Sending a regular message via a ForeignHandle.");

        self.send_message_foreign(message, None, &self.path).await
    }

    /// Sends a message to the actor and waits for a response.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self, message)))]
    async fn request(&self, message: M) -> Result<M::Response, ActorError> {
        #[cfg(all(feature = "tracing", debug_assertions))]
        event!(Level::TRACE, actor=self.path.to_string(), "Sending a request via a ForeignHandle.");

        // Create the responder
        let (responder, reciever) = oneshot::channel();

        // Send the message
        self.send_message_foreign(message, Some(responder), &self.path)
            .await?;

        #[cfg(all(feature = "tracing", debug_assertions))]
        event!(Level::TRACE, actor=self.path.to_string(), "Sending a request via a ForeignHandle.");

        // Await a response
        let res = reciever.await;

        #[cfg(all(feature = "tracing", debug_assertions))]
        event!(Level::TRACE, actor=self.path.to_string(), "ForeignHandle recieved response.");

        // Return the result with the error converted
        res.or(Err(ActorError::ForeignResponseFailed))
    }

    /// Sends a federated message to the referenced actor and does not wait for a response
    #[cfg(feature = "federated")]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self, message)))]
    async fn send_federated(&self, message: F) -> Result<(), ActorError> {
        #[cfg(all(feature = "tracing", debug_assertions))]
        event!(Level::TRACE, actor=self.path.to_string(), "Sending a federated message via a ForeignHandle.");

        self.send_federated_foreign(message, None, &self.path).await
    }

    /// Sends a federated message to the referenced actor and waits for a response.
    #[cfg(feature = "federated")]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self, message)))]
    async fn request_federated(&self, message: F) -> Result<F::Response, ActorError> {
        #[cfg(all(feature = "tracing", debug_assertions))]
        event!(Level::TRACE, actor=self.path.to_string(), "Sending a federated request via a ForeignHandle.");

        // Create the responder
        let (responder, reciever) = oneshot::channel();

        // Send the message
        self.send_federated_foreign(message, Some(responder), &self.path)
            .await?;

        #[cfg(all(feature = "tracing", debug_assertions))]
        event!(Level::TRACE, actor=self.path.to_string(), "ForeignHandle awaiting response for foreign request.");

        // Await a response
        let res = reciever.await;

        #[cfg(all(feature = "tracing", debug_assertions))]
        event!(Level::TRACE, actor=self.path.to_string(), "ForeignHandle recieved response to foreign request.");

        // Return the result with the error converted
        res.or(Err(ActorError::FederatedResponseFailed))
    }
}
