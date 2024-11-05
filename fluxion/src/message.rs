
use core::error::Error;

use slacktor::Message;

#[cfg(feature="serde")]
use crate::MessageID;

/// # [`MessageSendError`]
/// An error type that might be returned during a message send.
#[derive(Debug)]
#[non_exhaustive]
pub enum MessageSendError {
    #[cfg(feature = "serde")]
    SerializationError {
        message: alloc::string::String,
        source: alloc::boxed::Box<dyn core::error::Error>,
    },
    #[cfg(feature = "serde")]
    DeserializationError {
        message: alloc::string::String,
        source: alloc::boxed::Box<dyn core::error::Error>,
    },
    #[cfg(feature = "foreign")]
    DelegateError {
        message: alloc::string::String,
        source: alloc::boxed::Box<dyn core::error::Error>,
    },
    UnknownError(alloc::boxed::Box<dyn Error>),
}

impl core::fmt::Display for MessageSendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            #[cfg(feature = "serde")]
            MessageSendError::SerializationError { message, source: _ } => message.clone(),
            #[cfg(feature = "serde")]
            MessageSendError::DeserializationError { message, source: _ } => message.clone(),
            #[cfg(feature = "foreign")]
            MessageSendError::DelegateError { message, source: _ } => message.clone(),
            MessageSendError::UnknownError(e) => alloc::format!("{e}"),
        };

        write!(f, "MessageSendError: {message}")
    }
}

impl core::error::Error for MessageSendError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            #[cfg(feature = "serde")]
            Self::SerializationError { message: _, source } => Some(source.as_ref()),
            #[cfg(feature = "serde")]
            Self::DeserializationError { message: _, source } => Some(source.as_ref()),
            #[cfg(feature = "foreign")]
            Self::DelegateError { message: _, source } => Some(source.as_ref()),
            Self::UnknownError(e) => Some(e.as_ref()),
        }
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }
}

/// # [`IndeterminateMessage`]
/// An indeterminate message is a message for which it has not yet been determined whether it will be serialized.
/// Because of this, indeterminate messages require serde traits to be implemented, which is not the case with local messages.
#[cfg(feature = "serde")]
pub trait IndeterminateMessage: Message + MessageID + serde::Serialize + for<'a> serde::Deserialize<'a> 
where Self: Message + serde::Serialize + for<'a> serde::Deserialize<'a>,
    Self::Result: serde::Serialize + for<'a> serde::Deserialize<'a>{}

#[cfg(feature = "serde")]
impl<T> IndeterminateMessage for T
where T: Message + MessageID + serde::Serialize + for<'a> serde::Deserialize<'a>,
    Self::Result: serde::Serialize + for<'a> serde::Deserialize<'a> {}


/// # [`IndeterminateMessage`]
/// An indeterminate message is a message for which it has not yet been determined whether it will be serialized.
/// Because of this, indeterminate messages require serde traits to be implemented, which is not the case with local messages.
#[cfg(not(feature = "serde"))]
pub trait IndeterminateMessage: Message {}

#[cfg(not(feature = "serde"))]
impl<T: Message> IndeterminateMessage for T {}
