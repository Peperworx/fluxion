//! Contains [`ActorHandle`], a struct that is used for interacting with Actors, and other supporting types.

use tokio::sync::mpsc;

use crate::{message::{MessageType, Message, foreign::{ForeignMessage, ForeignReciever}, Notification}, error::ActorError};



/// # ActorHandle
/// [`ActorHandle`] provides a method through which to interact with actors.
pub struct ActorHandle<F, N, M>
where
    F: Message,
    N: Notification,
    M: Message, {
    
    /// The message sender for the actor
    pub(crate) message_sender: mpsc::Sender<MessageType<F, N, M>>,

    /// The foreign message sender for the actor
    pub(crate) foreign_sender: mpsc::Sender<ForeignMessage<F, N>>,

    /// The id of the actor
    pub(crate) id: String,
}



#[async_trait::async_trait]
impl<F, N, M> ForeignReciever for ActorHandle<F, N, M>
where
    F: Message,
    N: Notification,
    M: Message {

    type Federated = F;
    type Notification = N;

    
    async fn handle_foreign(&self, foreign: ForeignMessage<F, N>) -> Result<(), ActorError> {
        self.foreign_sender.send(foreign).await.or(Err(ActorError::ForeignSendFail))
    }
}

