//! Contains error types for the crate

use thiserror::Error;

/// Contains error policy handling systems.
/// # Note
/// This may be better as a separate crate.
pub mod policy;

/// # ActorError
/// An error type returned by an actor, as the result of failed communication with an actor,
/// or as the result of an internal error in actor initialization.
#[derive(Error, Debug, PartialEq, Clone)]
pub enum ActorError {
    #[error("A foreign message failed to send.")]
    ForeignSendFail,
    #[error("A foreign message failed to send a response")]
    ForeignRespondFailed,
    #[error("A foreign message failed to recieve a response")]
    ForeignResponseFailed,
    #[error("A foreign message responded with an unxepected type.")]
    ForeignResponseUnexpected,
    #[error("A foreign response failed to relay to the local sender")]
    ForeignResponseRelayFail,
    #[error("The target of a foreign message on this system was not found")]
    ForeignTargetNotFound,
    #[error("A message failed to be sent to a local actor")]
    MessageSendError,
    #[error("An actor's message channel was closed")]
    MessageChannelClosed,
    #[error("A message failed to recieve a response")]
    MessageResponseFailed,
    #[error("A federated message failed to recieve a response")]
    FederatedResponseFailed,
    #[error("There was an error handling a notification")]
    NotificationError,
}

/// # SystemError
/// An error type returned when an operation on a system fails.
#[derive(Error, Debug, PartialEq, Clone)]
pub enum SystemError {
    #[error("An actor with that id already exists")]
    ActorExists,
    #[error("An actor path was invalid")]
    InvalidPath,
}
