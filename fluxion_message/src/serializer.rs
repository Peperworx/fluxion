//! This module contains the [`MessageSerializer`] trait, which simplifies the serialization and deserialization of messages,
//! and enables different serialization systems to be implemented with very simple glue code.


use fluxion_error::MessageError;

use {
    alloc::vec::Vec,
    serde::{Deserialize, Serialize},
};

/// # `MessageSerializer`
/// This trait is used to simplify the serialization and deserialization of messages and their responses
pub trait MessageSerializer: 'static {
    /// Deserialize a message
    ///
    /// # Errors
    /// This function should only ever error with a [`FluxionError::DeserializeError`].
    fn deserialize<T: for<'a> Deserialize<'a>>(message: &[u8]) -> Result<T, MessageError>;

    /// Serialize a message
    ///
    /// # Errors
    /// This function should only ever error with a [`FluxionError::SerializeError`].
    fn serialize<T: Serialize>(message: T) -> Result<Vec<u8>, MessageError>;
}
