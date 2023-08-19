//! # Message
//! The [`Message`] trait encapsulates all Messages that can be sent between actors, including Notifications and Federated Messages.




#[cfg(serde)]
use {
    serde::{Serialize, Deserialize},
    crate::error::FluxionError,
    alloc::vec::Vec
};


/// # Message
/// This trait is used to mark Messages. Notifications are just Messages with a response type of `()`.
/// By default, all Messages and their responses must be [`Send`] + [`Sync`] + [`'static`].
pub trait Message: Send + Sync + 'static {
    /// The message's response
    type Response: Send + Sync + 'static;
}



/// # MessageSerializer
/// This trait is used to simplify the serialization and deserialization of messages and their responses
#[cfg(serde)]
pub trait MessageSerializer {

    /// Deserialize a message
    fn deserialize<T: for<'a> Deserialize<'a>, E>(message: Vec<u8>) -> Result<T, FluxionError<E>>;

    /// Serialize a message
    fn serialize<T: Serialize, E>(message: T) -> Result<Vec<u8>, FluxionError<E>>;
}