//! Contains the implementation of messages and surrounding types.

use std::any::Any;

/// Contains message handling traits that can be implemented by actors
pub mod handler;

/// Contains an implementation of a foreign message
pub mod foreign;

/// # Message
/// The [`Message`] trait should be implemented for any message, including Messages, and Federated Messages.
/// Both [`Message`] and [`Message::Response`] require messages to be [`Any`] + [`Send`] + [`Sync`] + `'static`.
/// Why require Any? Messages may be passed around as a `dyn Message` (as is the case with foreign channels).
/// These messages can be sent to an actor's supervisor, which knowing the type can try to downcast it.
pub trait Message: Any + Send + Sync + 'static {
    /// The response type of the message
    type Response: Any + Send + Sync + 'static;
}

/// # Notification
/// The [`Notification`] trait must be implemented for any notifications.
/// It requires notifications to be [`Send`] + [`Sync`] + ''static`.
/// This trait is automatically implemented for all types which implement its subtraits.
pub trait Notification: Send + Sync + 'static {}

impl<T> Notification for T where T: Send + Sync + 'static {}