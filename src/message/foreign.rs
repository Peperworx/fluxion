//! The implementation of foreign messages

use std::any::Any;

use tokio::sync::oneshot;

use super::Message;

/// # DynMessageResponse
/// Private type alias for dyn [`Any`] + [`Send`] + [`Sync`] + 'static
type DynMessageResponse = dyn Any + Send + Sync + 'static;


/// # ForeignMessage
/// The enum sent along a foreign message channel.
/// 
/// ## Generics
/// ForeignMessages contain variants [`ForeignMessage::FederatedMessage`] and [`ForeignMessage::Notification`],
/// which both contain their respective messages. Because Federated Messages and Notifications are uniform for an entire system,
/// they can be included as generics.
pub enum ForeignMessage<F: Message, N: super::Notification> {
    /// Contains a federated message sent to a foreign actor
    /// as well as its responder oneshot.
    FederatedMessage(F, Option<oneshot::Sender<F::Response>>),
    /// Contains a notification sent to a foreign actor
    Notification(N),
    /// Contains a message sent to a foreign actor
    Message(Box<dyn Message<Response = DynMessageResponse>>, Option<oneshot::Sender<Box<DynMessageResponse>>>)
}