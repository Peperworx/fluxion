//! Contains the implementation of messages and surrounding types.

use std::any::Any;

use tokio::sync::oneshot;

/// Contains message handling traits that can be implemented by actors
pub mod handler;

/// Contains an implementation of a foreign message
pub mod foreign;

/// # Message
/// The [`Message`] trait should be implemented for any message, including Messages, and Federated Messages.
/// Both [`Message`] and [`Message::Response`] require messages to be [`Any`] + [`Clone`] + [`Send`] + [`Sync`] + `'static`.
/// Why require Any? Sometimes messages may be passed around as a `dyn Any` (as is the case with foreign channels).
pub trait Message: Any + Clone + Send + Sync + 'static {
    /// The response type of the message
    type Response: Any + Clone + Send + Sync + 'static;
}


/// # Notification
/// The [`Notification`] trait must be implemented for any notifications.
/// It requires notifications to be [`Send`] + [`Sync`] + ''static`.
/// This trait is automatically implemented for all types which implement its subtraits.
pub trait Notification: Clone + Send + Sync + 'static {}

impl<T> Notification for T where T: Clone + Send + Sync + 'static {}


/// # DynMessageResponse
/// Internal type alias for dyn [`Any`] + [`Send`] + [`Sync`] + 'static
pub(crate) type DynMessageResponse = dyn Any + Send + Sync + 'static;


/// # MessageType
/// An enum that contains each different type of message sent to an actor.
/// Used to reduce the nuber of mpsc channels required.
pub enum MessageType<F: Message, N, M: Message> {
    /// A federated message
    Federated(F, Option<oneshot::Sender<F::Response>>),
    /// A notification
    Notification(N),
    /// A message
    Message(M, Option<oneshot::Sender<M::Response>>)
}