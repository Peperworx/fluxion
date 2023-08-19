//! # Message
//! The [`Message`] trait encapsulates all Messages that can be sent between actors, including Notifications and Federated Messages.



use alloc::vec::Vec;

use crate::actor::Handle;

#[cfg(serde)]
use serde::{Serialize, Deserialize};

use crate::{error::FluxionError, actor::Actor};

#[cfg(foreign)]
pub mod foreign;




/// # Message
/// This trait is used to mark Messages. Notifications are just Messages with a response type of `()`.
/// By default, all Messages and their responses must be [`Send`] + [`Sync`] + [`'static`].
pub trait Message: Send + Sync + 'static {
    /// The message's response
    type Response: Send + Sync + 'static;

    /// The error that may be returned when handling this message
    type Error: Send + Sync + 'static;
}



