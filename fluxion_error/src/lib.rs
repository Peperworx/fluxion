//! # Fluxion Error
//! 
//! Error types used by Fluxion.


// The following will be in *every* crate related to fluxion.
#![no_std]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use thiserror_no_std::Error;

/// # [`ActorError`]
/// The error type returned by [`Actor`]s, which allows for a custom error type via generics.
#[derive(Error, Debug)]
pub enum ActorError<E> {
    #[error("custom error from actor")]
    CustomError(E),
    #[error("a message error was received")]
    MessageError(#[from] MessageError),
}

/// # [`MessageError`]
/// An error that arises from a message failing to send.
#[derive(Error, Debug)]
pub enum MessageError {
    #[cfg(serde)]
    #[error("error deserializing a foreign message")]
    DeserializeError,
    #[cfg(serde)]
    #[error("error serializing a foreign message")]
    SerializeError,
    #[error("message response failed")]
    ResponseFailed,
    #[error("message failed to send")]
    SendError,
}

/// # [`SystemError`]
/// An operation performed on the system failed
#[derive(Error, Debug)]
pub enum SystemError {
    #[error("the added actor already exists")]
    ActorExists,
}