//! Contains the implementation of foreign messages

use super::Message;

use alloc::vec::Vec;

/// Contains a Vec<u8>, and implements Message, with a response of Vec<u8>.
/// This can be deserialized to another message type, depending on the 
pub struct ForeignMessage(Vec<u8>);

impl Message for ForeignMessage {
    type Response = Vec<u8>;
}