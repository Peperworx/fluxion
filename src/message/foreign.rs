//! The implementation of foreign messages


use tokio::sync::oneshot;

use crate::actor::path::ActorPath;

use super::{Message, DynMessageResponse};



/// # ForeignMessage
/// The enum sent along a foreign message channel.
/// 
/// ## Generics
/// ForeignMessages contain variants [`ForeignMessage::FederatedMessage`] and [`ForeignMessage::Notification`],
/// which both contain their respective messages. Because Federated Messages and Notifications are uniform for an entire system,
/// they can be included as generics.
#[derive(Debug)]
pub enum ForeignMessage<F: Message, N: super::Notification> {
    /// Contains a federated message sent to a foreign actor
    /// as well as its responder oneshot and target
    FederatedMessage(F, Option<oneshot::Sender<F::Response>>, ActorPath),
    /// Contains a notification sent to a foreign actor
    Notification(N),
    /// Contains a message sent to a foreign actor as well as it's responder and target
    Message(Box<DynMessageResponse>, Option<oneshot::Sender<Box<DynMessageResponse>>>, ActorPath)
}