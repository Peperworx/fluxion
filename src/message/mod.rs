//! Contains the implementation of messages and surrounding types.

/// Contains message handling traits that can be implemented by actors
pub mod handler;

/// # Message
/// The [`Message`] trait should be implemented for any message, including Messages, and Federated Messages.
/// Both [`Message`] and [`Message::Response`] require messages to be Send + Sync + 'static.
pub trait Message: Send + Sync + 'static {
    /// The response type of the message
    type Response: Send + Sync + 'static;
}