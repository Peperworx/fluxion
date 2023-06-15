//! The implementation of foreign messages


use tokio::sync::oneshot;

use crate::{actor::path::ActorPath, error::ActorError};

use super::{Message, DynMessageResponse, Notification};



/// # ForeignMessage
/// The enum sent along a foreign message channel.
/// 
/// ## Generics
/// ForeignMessages contain variants [`ForeignMessage::FederatedMessage`] and [`ForeignMessage::Notification`],
/// which both contain their respective messages. Because Federated Messages and Notifications are uniform for an entire system,
/// they can be included as generics.
#[derive(Debug)]
pub enum ForeignMessage<F: Message, N: Notification> {
    /// Contains a federated message sent to a foreign actor
    /// as well as its responder oneshot and target
    FederatedMessage(F, Option<oneshot::Sender<F::Response>>, ActorPath),
    /// Contains a notification sent to a foreign actor
    Notification(N),
    /// Contains a message sent to a foreign actor as well as it's responder and target
    Message(Box<DynMessageResponse>, Option<oneshot::Sender<Box<DynMessageResponse>>>, ActorPath)
}

/// # ForeignMessenger
/// [`ForeignMessenger`] is a trait that can be implemented on any type that is used to send messages to a foreign actor.
/// This is used to abstract over actors stored in a system so that it is not necessary to know the type of their messages.
#[async_trait::async_trait]
pub trait ForeignMessenger {

    /// The type of the federated message that is sent by both the local and foreign system
    type Federated: Message;

    /// The type of the notification that is sent by both the local and foreign system
    type Notification: Notification;

    /// This function must be implemented by every [`ForeignMessenger`]. It sends the passed foreign
    /// message to the foreign actor.
    async fn send_raw_foreign(&self, message: ForeignMessage<Self::Federated, Self::Notification>) -> Result<(), ActorError>;

    /// This function must be implemented by every [`ForeignMessenger]`
    /// It must return true if someone is ready to recieve a foreign message
    fn can_send_foreign(&self) -> bool;

    /// This function is automatically implemented. It takes a local notification,
    /// and sends it using [`ForeignMessenger::send_raw_foreign`]
    async fn send_notification_foreign(&self, notification: Self::Notification) -> Result<(), ActorError> {
        // Create the foreign message
        let foreign = ForeignMessage::Notification(notification);

        // Send the foreign message
        self.send_raw_foreign(foreign).await
    }
    
    /// This function is automatically implemented. It takes a local message, and
    /// sends it using [`ForeignMessenger::send_raw_foreign`]
    async fn send_message_foreign<M: Message>(&self,
        message: M, responder: Option<oneshot::Sender<M::Response>>,
        target_actor: ActorPath) -> Result<(), ActorError> {

            // If we should wait for a response, then do so
        let (foreign_responder, responder_recieve) = if responder.is_some() {
            let channel = oneshot::channel();
            (Some(channel.0), Some(channel.1))
        } else {
            (None, None)
        };

        // Put the message into a foreign message
        let foreign = ForeignMessage::<Self::Federated, Self::Notification>::Message(Box::new(message), foreign_responder, target_actor);

        // If someone is listening for a foreign message, then send the foreign message
        if self.can_send_foreign() {
            self.send_raw_foreign(foreign).await?;
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

    /// This function is automatically implemented. It takes a local federated message, and
    /// sends it using [`ForeignMessenger::send_raw_foreign`]
    async fn send_federated_foreign(&self, federated: Self::Federated,
        responder: Option<oneshot::Sender<<Self::Federated as Message>::Response>>,
        target_actor: ActorPath) -> Result<(), ActorError> {
        // Create the foreign message
        let foreign = ForeignMessage::FederatedMessage(federated, responder, target_actor);
        
        // Send the foreign message
        self.send_raw_foreign(foreign).await
    }

}