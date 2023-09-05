//! This module contains the [`MessageSerializer`] trait, which simplifies the serialization and deserialization of messages,
//! and enables different serialization systems to be implemented with very simple glue code.

use crate::error::FluxionError;
use {
    alloc::vec::Vec,
    serde::{Deserialize, Serialize},
};

/// # `MessageSerializer`
/// This trait is used to simplify the serialization and deserialization of messages and their responses
pub trait MessageSerializer {
    /// Deserialize a message
    ///
    /// # Errors
    /// This function should only ever error with a [`FluxionError::DeserializeError`].
    fn deserialize<T: for<'a> Deserialize<'a>, E>(message: &[u8]) -> Result<T, FluxionError<E>>;

    /// Serialize a message
    ///
    /// # Errors
    /// This function should only ever error with a [`FluxionError::SerializeError`].
    fn serialize<T: Serialize, E>(message: T) -> Result<Vec<u8>, FluxionError<E>>;
}
