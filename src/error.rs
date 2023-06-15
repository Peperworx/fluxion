//! Contains error types for the crate

use thiserror::Error;

/// # ActorError
/// An error type returned by an actor, as the result of failed communication with an actor,
/// or as the result of an internal error in Actor initialization.
#[derive(Error, Debug)]
pub enum ActorError {
    #[error("A foreign message failed to send.")]
    ForeignSendFail,
    #[error("A foreign message failed to send a response")] 
    ForeignRespondFail,
    #[error("A foreign message responded with an unxepected type.")]
    ForeignResponseUnexpected,
    #[error("A foreign response failed to relay to the local sender")]
    ForeignResponseRelayFail,
}