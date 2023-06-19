//! Contains [`ActorHandle`], a struct that is used for interacting with Actors, and other supporting types.

use tokio::sync::mpsc;

use crate::{message::{LocalMessage, Message, foreign::{ForeignMessage, ForeignReciever}, Notification, DualMessage}, error::ActorError};



/// # ActorHandle
/// [`ActorHandle`] provides a method through which to interact with actors.
#[derive(Clone)]
pub struct ActorHandle<F, M>
where
    F: Message,
    M: Message, {
    
    /// The message sender for the actor
    pub(crate) message_sender: mpsc::Sender<DualMessage<F, M>>,

    /// The id of the actor
    pub(crate) id: String,
}

impl<F, M> ActorHandle<F, M>
where
    F: Message,
    M: Message, {
    
    /// Gets the actor's id
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Sends a raw message to the actor
    pub async fn send_raw_message(&self, message: LocalMessage<F, M>) -> Result<(), ActorError> {
        self.message_sender.send(DualMessage::LocalMessage(message)).await.or(Err(ActorError::MessageSendError))
    }
}


#[async_trait::async_trait]
impl<F, M> ForeignReciever for ActorHandle<F, M>
where
    F: Message,
    M: Message {

    type Federated = F;

    async fn handle_foreign(&self, foreign: ForeignMessage<F>) -> Result<(), ActorError> {
        self.message_sender.send(DualMessage::ForeignMessage(foreign)).await.or(Err(ActorError::ForeignSendFail))
    }
}

