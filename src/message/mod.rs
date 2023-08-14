//! # Message
//! The [`Message`] trait encapsulates all Messages that can be sent between actors, including Notifications and Federated Messages.

use alloc::vec::Vec;
#[cfg(serde)]
use serde::{Deserialize, Serialize, Serializer, Deserializer};

use crate::{actor::{Handle, Actor}, error::FluxionError};


#[cfg(foreign)]
pub mod foreign;


/// # Message
/// This trait is used to mark Messages. Notifications are just Messages with a response type of `()`.
/// By default, all Messages and their responses must be [`Send`] + [`Sync`] + [`'static`].
pub trait Message: Send + Sync + 'static {
    /// The message's response
    type Response: Send + Sync + 'static;
}
