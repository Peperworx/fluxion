use std::sync::Arc;

use tokio::sync::{mpsc, Mutex, broadcast};

use crate::{message::foreign::ForeignMessage, Message};


/// # ForeignComponents
/// Components of System that are only used when foreign actor support is enabled.
#[derive(Clone, Debug)]
pub struct ForeignComponents<F, N>
where
    F: Message {
    /// The reciever for foreign messages
    /// This uses a Mutex to provide interior mutability.
    pub foreign_reciever: Arc<Mutex<Option<mpsc::Receiver<ForeignMessage<F>>>>>,

    /// The sender for foreign messages
    pub foreign_sender: mpsc::Sender<ForeignMessage<F>>,

    /// The foreign notification sender
    pub foreign_notification: broadcast::Sender<N>,
}