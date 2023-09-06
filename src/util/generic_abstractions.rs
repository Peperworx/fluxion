//! # Generic Abstractions
//! This file contains a way to abstract many different generics and feature flags into a singular, concise generic
//! that uses associated types to store the values.
//! This method significantly reduces repetition and makes code much easier to read.
//!
//! ## [`MessageParams`]
//! This is the basic generic abstraction. It does not depend on any other generic abstractions, and it contains
//! only two associated types: `Federated` and `Notification`, which only exist if their respective feature flags are enabled.

use crate::message::Message;

/// # [`MessageParams`]
/// The simplest generic abstraction, containing the federated message and notification types.
pub trait MessageParams {
    /// The federated message associated with the system
    #[cfg(federated)]
    type Federated: Message;

    /// The notification associated with the system
    #[cfg(notification)]
    type Notification: Message;
}

/// # [`ActorParams`]
/// Parameters that are specific to a single actor. This includes the [`MessageParams`], the actor's [`Message`],
/// and the error type that the actor can return.
pub trait ActorParams {

    /// The message that the actor can handle
    type Message: Message;

    /// The [`MessageParams`] that this actor needs to support.
    /// This is named [`SystemMessages`] because it contains message types uniform
    /// across the entire system
    type SystemMessages: MessageParams;

    /// The error type that this actor may return
    
}

