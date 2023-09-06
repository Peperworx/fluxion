//! # Generic Abstractions
//! This file contains a way to abstract many different generics and feature flags into a singular, concise generic
//! that uses associated types to store the values.
//! This method significantly reduces repetition and makes code much easier to read.
//!
//! ## [`MessageParams`]
//! This is the basic generic abstraction. It does not depend on any other generic abstractions, and it contains
//! only two associated types: `Federated` and `Notification`, which only exist if their respective feature flags are enabled.
//!
//! ## [`ActorParams`]
//! This contains both the contents of [`MessageParams`], a [`Message`] type, and the type of the actor that this parameter stores info for.
//! This is used to provide generics for the actor's supervisor.


use crate::{message::Message, actor::{Actor, Handle}};

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
/// and of course the actor's specific type.
pub trait ActorParams {

    /// The message that the actor can handle
    type Message: Message;

    /// The [`MessageParams`] that this actor needs to support.
    /// This is named [`SystemMessages`] because it contains message types uniform
    /// across the entire system
    type SystemMessages: MessageParams;

    
    cfg_if::cfg_if! {
        if #[cfg(all(notification, federated))] {
            /// The type of the actor itself.
            type Actor: Actor + Handle<Self::Message> + Handle<<Self::SystemMessages as MessageParams>::Notification> + Handle<<Self::SystemMessages as MessageParams>::Federated>;
        } else if #[cfg(notification)] {
            /// The type of the actor itself.
            type Actor: Actor + Handle<Self::Message> + Handle<<Self::SystemMessages as MessageParams>::Notification>;
        } else if #[cfg(federated)] {
            /// The type of the actor itself.
            type Actor: Actor + Handle<Self::Message> + Handle<<Self::SystemMessages as MessageParams>::Federated>;
        } else {
            /// The type of the actor itself.
            type Actor: Actor + Handle<<Self::Messages as MessageParams>::Message>;
        }
    }
}

