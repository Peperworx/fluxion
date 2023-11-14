//! # Errors
//! Contains different error types used throughout Fluxion.
use thiserror_no_std::Error;

#[cfg(foreign)]
use serde::{Serialize, Deserialize};

/// # [`ActorError`]
/// The error type returned by [`Actor`]s, which allows for a custom error type via generics.
#[derive(Error, Debug, PartialEq)]
#[cfg_attr(foreign, derive(Serialize, Deserialize))]
pub enum ActorError<E> {
    #[error("CustomError: `{0:?}`")]
    CustomError(E),
    #[error("actor supervisor failed to receive a message")]
    MessageReceiveError,
    
    #[error("message sent over channel failed to downcast")]
    InvalidMessageType,
    #[error("SendError: {0:?}")]
    SendError(#[from] SendError)
}

/// # [`SendError`]
/// The error type returned when sending a message to an actor
#[derive(Error, Debug, PartialEq)]
#[cfg_attr(foreign, derive(Serialize, Deserialize))]
pub enum SendError {
    #[error("no response was returned")]
    NoResponse,
    #[error("failed to send a response")]
    ResponseFailed,
    #[error("failed to serialize a message")]
    SerializationFailed,
}

/// # [`ForeignError`]
/// Errors generated when handling a foreign message
#[cfg(foreign)]
#[derive(Error, Debug)]
pub enum ForeignError {
    #[error("the target's system does not match")]
    SystemNoMatch,
    #[error("there is no actor with the matching id")]
    NoActor,
}