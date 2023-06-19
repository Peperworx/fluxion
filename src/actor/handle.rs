//! Contains [`ActorHandle`], a struct that is used for interacting with Actors, and other supporting types.

use tokio::sync::mpsc;

use crate::{message::{MessageType, Message, foreign::{ForeignMessage, ForeignReciever}, Notification, DualMessage}, error::ActorError};



/// # ActorHandle
/// [`ActorHandle`] provides a method through which to interact with actors.
#[derive(Clone)]
pub struct ActorHandle<F, N, M>
where
    F: Message,
    N: Notification,
    M: Message, {
    
    /// The message sender for the actor
    pub(crate) message_sender: mpsc::Sender<DualMessage<F, M>>,

    /// The id of the actor
    pub(crate) id: String,
}

impl<F, N, M> ActorHandle<F, N, M>
where
    F: Message,
    N: Notification,
    M: Message, {
    
    /// Gets the actor's id
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Sends a raw message to the actor
    pub async fn send_raw_message(&self, message: MessageType<F, M>) -> Result<(), ActorError> {
        self.message_sender.send(DualMessage::MessageType(message)).await.or(Err(ActorError::MessageSendError))
    }
}


#[async_trait::async_trait]
impl<F, N, M> ForeignReciever for ActorHandle<F, N, M>
where
    F: Message,
    N: Notification,
    M: Message {

    type Federated = F;

    async fn handle_foreign(&self, foreign: ForeignMessage<F>) -> Result<(), ActorError> {
        self.message_sender.send(DualMessage::ForeignMessage(foreign)).await.or(Err(ActorError::ForeignSendFail))
    }
}

