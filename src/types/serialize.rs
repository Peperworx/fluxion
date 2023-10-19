//! Defines a trait that provides methods for serializing and deserializing a message.

use serde::{Deserialize, Serialize};

use alloc::vec::Vec;

/// # [`MessageSerializer`]`
/// This trait is used to simplify the serialization and deserialization of messages and their responses
#[cfg(serde)]
pub trait MessageSerializer {

    /// Deserialize a message, returns [`None`] in case of error.
    fn deserialize<T: for<'a> Deserialize<'a>>(message: Vec<u8>) -> Option<T>;

    /// Serialize a message, returns [`None`] in case of error.
    fn serialize<T: Serialize>(message: T) -> Option<Vec<u8>>;
}