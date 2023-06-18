//! Contains error types for the crate

use thiserror::Error;


/// Contains error policy handling systems.
/// # Todo
/// This may be better as a separate crate.
pub mod policy;


/// # ActorError
/// An error type returned by an actor, as the result of failed communication with an actor,
/// or as the result of an internal error in Actor initialization.
#[derive(Error, Debug, PartialEq, Clone)]
pub enum ActorError {
    #[error("A foreign message failed to send.")]
    ForeignSendFail,
    #[error("A foreign message failed to send a response")] 
    ForeignRespondFail,
    #[error("A foreign message responded with an unxepected type.")]
    ForeignResponseUnexpected,
    #[error("A foreign response failed to relay to the local sender")]
    ForeignResponseRelayFail,
    #[error("The target of a foreign message on this system was not found")]
    ForeignTargetNotFound,
}


/// # SystemError
/// An error type returned when an operation on a system fails.
#[derive(Error, Debug, PartialEq, Clone)]
pub enum SystemError {
    #[error("An actor with that id already exists")]
    ActorExists,
}