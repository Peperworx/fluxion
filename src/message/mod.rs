//! Contains the implementation of messages and surrounding types.

use std::any::Any;

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


