//! Contains [`ActorHandle`], a struct that is used for interacting with Actors, and other supporting types.

use std::any::Any;

use tokio::sync::{mpsc, oneshot};

use crate::{
    error::ActorError,
    message::{
        foreign::{ForeignMessage, ForeignMessenger, ForeignReciever},
        DualMessage, LocalMessage, Message, Notification,
    },
    system::System,
};

use super::{path::ActorPath, ActorEntry};

/// # ActorHandle
/// [`ActorHandle`] is a trait which provides methods through which to interface with actorsd
#[async_trait::async_trait]
pub trait ActorHandle<F, M>: Send + Sync + 'static
where
    F: Message,
    M: Message,
{
    /// Gets the actor's path
    fn get_path(&self) -> &ActorPath;

    /// Sends a message to the actor
    async fn send(&self, message: M) -> Result<(), ActorError>;

    /// Sends a message to the actor and awaits a response
    async fn request(&self, message: M) -> Result<M::Response, ActorError>;

    /// Sends a federated message to the actor
    async fn send_federated(&self, message: F) -> Result<(), ActorError>;

    /// Sends a federated message to the actor and awaits a response
    async fn request_federated(&self, message: F) -> Result<F::Response, ActorError>;
}

/// # LocalHandle
/// [`LocalHandle`] acts as an [`ActorHandle`] for local actors
#[derive(Clone)]
pub struct LocalHandle<F, M>
where
    F: Message,
    M: Message,
{
    /// The message sender for the actor
    pub(crate) message_sender: mpsc::Sender<DualMessage<F, M>>,

    /// The id of the actor
    pub(crate) path: ActorPath,
}

impl<F, M> LocalHandle<F, M>
where
    F: Message,
    M: Message,
{
    /// Sends a raw message to the actor
    async fn send_raw_message(&self, message: LocalMessage<F, M>) -> Result<(), ActorError> {
        self.message_sender
            .send(DualMessage::LocalMessage(message))
            .await
            .or(Err(ActorError::MessageSendError))
    }
}

#[async_trait::async_trait]
impl<F, M> ActorHandle<F, M> for LocalHandle<F, M>
where
    F: Message,
    M: Message,
{
    /// Gets the actor's id
    fn get_path(&self) -> &ActorPath {
        &self.path
    }

    /// Sends a message to the actor
    async fn send(&self, message: M) -> Result<(), ActorError> {
        self.send_raw_message(LocalMessage::<F, M>::Message(message, None))
            .await
    }

    /// Sends a message to the actor and awaits a response
    async fn request(&self, message: M) -> Result<M::Response, ActorError> {
        // Create the responder
        let (responder, reciever) = oneshot::channel();

        // Send the message
        self.send_raw_message(LocalMessage::<F, M>::Message(message, Some(responder)))
            .await?;

        // Await a response
        let res = reciever.await;

        // Return the result with the error converted
        res.or(Err(ActorError::MessageResponseFailed))
    }

    /// Sends a federated message to the actor
    async fn send_federated(&self, message: F) -> Result<(), ActorError> {
        self.send_raw_message(LocalMessage::<F, M>::Federated(message, None))
            .await
    }

    /// Sends a federated message to the actor and awaits a response
    async fn request_federated(&self, message: F) -> Result<F::Response, ActorError> {
        // Create the responder
        let (responder, reciever) = oneshot::channel();

        // Send the message
        self.send_raw_message(LocalMessage::<F, M>::Federated(message, Some(responder)))
            .await?;

        // Await a response
        let res = reciever.await;

        // Return the result with the error converted
        res.or(Err(ActorError::FederatedResponseFailed))
    }
}

#[async_trait::async_trait]
impl<F, M> ForeignReciever for LocalHandle<F, M>
where
    F: Message,
    M: Message,
{
    type Federated = F;

    async fn handle_foreign(&self, foreign: ForeignMessage<F>) -> Result<(), ActorError> {
        self.message_sender
            .send(DualMessage::ForeignMessage(foreign))
            .await
            .or(Err(ActorError::ForeignSendFail))
    }
}

impl<F, M> ActorEntry for LocalHandle<F, M>
where
    F: Message,
    M: Message,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// # ForeignHandle
/// [`ForeignHandle`] serves as an [`ActorHandle`] for foreign actors.
pub struct ForeignHandle<F: Message, N: Notification> {
    /// The foreign channel
    pub(crate) foreign: mpsc::Sender<ForeignMessage<F>>,
    /// The system
    pub(crate) system: System<F, N>,
    /// The path of the foreign actor
    pub(crate) path: ActorPath,
}

#[async_trait::async_trait]
impl<F: Message, N: Notification> ForeignMessenger for ForeignHandle<F, N> {
    /// The type of the federated message that is sent by both the local and foreign system
    type Federated = F;

    /// This function must be implemented by every [`ForeignMessenger`]. It sends the passed foreign
    /// message to the foreign actor.
    async fn send_raw_foreign(
        &self,
        message: ForeignMessage<Self::Federated>,
    ) -> Result<(), ActorError> {
        self.foreign
            .send(message)
            .await
            .or(Err(ActorError::ForeignSendFail))
    }

    /// This function must be implemented by every [`ForeignMessenger]`
    /// It must return true if someone is ready to recieve a foreign message
    async fn can_send_foreign(&self) -> bool {
        self.system.can_send_foreign().await
    }
}

#[async_trait::async_trait]
impl<F: Message, M: Message, N: Notification> ActorHandle<F, M> for ForeignHandle<F, N> {
    /// Gets the actor's id
    fn get_path(&self) -> &ActorPath {
        &self.path
    }

    /// Sends a message to the actor
    async fn send(&self, message: M) -> Result<(), ActorError> {
        self.send_message_foreign(message, None, &self.path).await
    }

    /// Sends a message to the actor and awaits a response
    async fn request(&self, message: M) -> Result<M::Response, ActorError> {
        // Create the responder
        let (responder, reciever) = oneshot::channel();

        // Send the message
        self.send_message_foreign(message, Some(responder), &self.path)
            .await?;

        // Await a response
        let res = reciever.await;

        // Return the result with the error converted
        res.or(Err(ActorError::ForeignResponseFailed))
    }

    /// Sends a federated message to the actor
    async fn send_federated(&self, message: F) -> Result<(), ActorError> {
        self.send_federated_foreign(message, None, &self.path).await
    }

    /// Sends a federated message to the actor and awaits a response
    async fn request_federated(&self, message: F) -> Result<F::Response, ActorError> {
        // Create the responder
        let (responder, reciever) = oneshot::channel();

        // Send the message
        self.send_federated_foreign(message, Some(responder), &self.path)
            .await?;

        // Await a response
        let res = reciever.await;

        // Return the result with the error converted
        res.or(Err(ActorError::FederatedResponseFailed))
    }
}
