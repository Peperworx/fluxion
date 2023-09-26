//! # Errors
//! Contains different error types used throughout Fluxion.
use thiserror_no_std::Error;

/// # [`ActorError`]
/// The error type returned by [`Actor`]s, which allows for a custom error type via generics.
#[derive(Error, Debug)]
pub enum ActorError<E> {
    #[error("custom error from actor")]
    CustomError(E),
}
