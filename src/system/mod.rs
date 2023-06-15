//! The implementation of systems and surrounding types

use std::{sync::Arc, mem};

use tokio::sync::{mpsc, Mutex, oneshot};

use crate::{message::{foreign::ForeignMessage, Notification, Message}, error::ActorError, actor::path::ActorPath};


/// # System
/// The core part of Fluxion, the [`System`] runs actors and handles communications between other systems.
/// 
/// ## Inter-System Communication
/// Fluxion systems enable communication by having what is called a foreign channel.
/// The foreign channel is an mpsc channel, the Reciever for which can be retrieved once by a single outside source using [`System::get_foreign`].
/// When a Message or Foreign Message is sent to an external actor, or a Notification is sent at all, the foreign
/// channel will be notified.
pub struct System<F, N>
where
    F: Message,
    N: Notification, {

    /// The reciever for foreign messages
    /// This uses a Mutex to provide interior mutability.
    foreign_reciever: Arc<Mutex<Option<mpsc::Receiver<ForeignMessage<F, N>>>>>,
    
    /// The sender for foreign messages
    foreign_sender: mpsc::Sender<ForeignMessage<F, N>>,
}

impl<F, N> System<F, N>
where
    F: Message,
    N: Notification {

    /// Creates a new system
    pub fn new() -> Self {
        // Create the foreign channel
        let (foreign_sender, foreign_reciever) = mpsc::channel(16);

        Self {
            foreign_reciever: Arc::new(Mutex::new(Some(foreign_reciever))),
            foreign_sender
        }
    }
    
    /// Returns the foreign channel reciever wrapped in an [`Option<T>`].
    /// [`None`] will be returned if the foreign reciever has already been retrieved.
    pub async fn get_foreign(&self) -> Option<mpsc::Receiver<ForeignMessage<F, N>>> {
        
        // Lock the foreign reciever
        let mut foreign_reciever = self.foreign_reciever.lock().await;

        // Return the contents and replace with None
        mem::take(std::ops::DerefMut::deref_mut(&mut foreign_reciever))
    }

    /// Forces a normal Message to be sent as a foreign message
    pub async fn force_foreign_send_message<M: Message>(&self, message: M, responder: Option<oneshot::Sender<M::Response>>, target_actor: ActorPath) -> Result<(), ActorError> {
        
        // If we should wait for a response, then do so
        let (foreign_responder, responder_recieve) = if responder.is_some() {
            let channel = oneshot::channel();
            (Some(channel.0), Some(channel.1))
        } else {
            (None, None)
        };

        // Put the message into a foreign message
        let foreign = ForeignMessage::<F, N>::Message(Box::new(message), foreign_responder, target_actor);

        // If the foreign reciever is None (which means that someone is listening for a foreign message), then send the foreign message
        if self.foreign_reciever.lock().await.is_none() {
            self.foreign_sender.send(foreign).await.or(Err(ActorError::ForeignSendFail))?;
        } else {
            return Ok(());
        }
        
        // If we should wait for a response, then do so
        if let (Some(target), Some(source)) = (responder, responder_recieve) {
            // Wait for the foreign response
            let res = source.await.or(Err(ActorError::ForeignRespondFail))?;

            // Downcast
            let res = res.downcast_ref::<M::Response>().ok_or(ActorError::ForeignResponseUnexpected)?;

            // Relay the response
            target.send(res.clone()).or(Err(ActorError::ForeignResponseRelayFail))?;
        }

        Ok(())
    }

    /// Forces a notification to be sent as a foreign message
    pub async fn force_foreign_send_notification(&self, notification: N) -> Result<(), ActorError> {
        // Create the foreign message
        let foreign = ForeignMessage::Notification(notification);

        // Send the foreign message
        self.foreign_sender.send(foreign).await.or(Err(ActorError::ForeignSendFail))
    }

    /// Forces a federated message to be send as a foreign message
    pub async fn force_foreign_send_federated(&self, federated: F, responder: Option<oneshot::Sender<F::Response>>, target_actor: ActorPath) -> Result<(), ActorError> {

        // Create the foreign message
        let foreign = ForeignMessage::FederatedMessage(federated, responder, target_actor);

        // Send the foreign message
        self.foreign_sender.send(foreign).await.or(Err(ActorError::ForeignSendFail))
    }
}