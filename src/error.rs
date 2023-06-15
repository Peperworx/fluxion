//! Contains error types for the crate

use thiserror::Error;

/// # ActorError
/// An error type returned by an actor, as the result of failed communication with an actor,
/// or as the result of an internal error in Actor initialization.
#[derive(Error, Debug)]
pub enum ActorError {

}