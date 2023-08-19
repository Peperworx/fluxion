//! # Message
//! The [`Message`] trait encapsulates all Messages that can be sent between actors, including Notifications and Federated Messages.



use alloc::vec::Vec;

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

    /// The serde dispatcher.
    /// Only required when serde is enabled.
    #[cfg(serde)]
    type Dispatcher: SerdeDispatcher;


    #[cfg(serde)]
    fn dispatch_serialized(actor: &dyn Actor<Error = Self::Error>, message: Vec<u8>) -> Result<Vec<u8>, FluxionError<Self::Error>>
    where
        Self: Sized
    {
        Self::Dispatcher::dispatch_serialized::<Self, Self::Error>(actor, message)
    }
}



/// # SerdeDispatcher
/// This trait provides a method through which to dispatch serialized messages while leveraging type erasure.
#[cfg(serde)]
pub trait SerdeDispatcher {
    /// Dispatch a serialized message
    fn dispatch_serialized<M: Message, E>(actor: &dyn Actor<Error = E>, message: Vec<u8>) -> Result<Vec<u8>, FluxionError<E>>;
}