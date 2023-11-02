//! # Event
//! Wraps a message and associated metadata that is available to the actor receiving the message.

use crate::{ActorId, Message};

/// # [`Event`]
/// Wraps a message and associated metadata.
#[derive(Clone, Debug)]
pub struct Event<M: Message> {
    /// The wrapped message
    pub message: M,
    /// The message's source. None if the message was sent outside of any specific actor.
    pub source: Option<ActorId>,
    /// The message's original target.
    pub target: ActorId,
}