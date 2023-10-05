//! # Errors
//! Contains different error types used throughout Fluxion.
use thiserror_no_std::Error;

/// # [`ActorError`]
/// The error type returned by [`Actor`]s, which allows for a custom error type via generics.
#[derive(Error, Debug)]
pub enum ActorError<E> {
    #[error("custom error from actor")]
    CustomError(E),
    #[error("actor supervisor failed to receive a message")]
    MessageReceiveError,
    #[error("message handler failed to send the response")]
    ResponseFailed,
    #[error("message sent over channel failed to downcast")]
    InvalidMessageType
}

/// # [`SendError`]
/// The error type returned when sending a message to an actor
#[derive(Error, Debug)]
pub enum SendError {
    
}