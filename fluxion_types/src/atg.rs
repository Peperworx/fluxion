//! # Associated Type Generics
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


use fluxion_message::Message;

use crate::{executor::Executor, Actor, Handle};
#[cfg(serde)]
use crate::message::serializer::MessageSerializer;

/// # [`MessageParams`]
/// The simplest generic abstraction, containing the federated message and notification types.
pub trait MessageParams: 'static {
    /// The federated message associated with the system
    #[cfg(federated)]
    type Federated: Message;

    /// The notification associated with the system
    #[cfg(notification)]
    type Notification: Message + Clone;
}

/// # [`SystemParams`]
/// Parameters specific to an entire system, including the [`MessageParams`], and the [`MessageSerializer`]
pub trait SystemParams: 'static {
    /// The [`MessageParams`] associated with this system.
    /// This is named [`SystemMessages`] because it contains message types uniform
    /// across the entire system
    #[cfg(any(federated, notification))]
    type SystemMessages: MessageParams;

    /// The [`MessageSerializer`] used for foreign messages across the system
    #[cfg(serde)]
    type Serializer: MessageSerializer;

    /// The [`Executor`] that tasks are spawned on.
    type Executor: Executor;
}

/// # [`ActorParams`]
/// Parameters that are specific to a single actor. This includes the [`MessageParams`], and the actor's [`Message`] type.
/// The [`SystemParams`] are provided as a generic to reduce long and complex associated type clairifications.
pub trait ActorParams<S: SystemParams>: 'static {
    /// The message that the actor can handle
    type Message: Message;

    cfg_if::cfg_if! {
        if #[cfg(all(notification, federated))] {
            /// The type of the actor itself.
            type Actor: Actor
                + Handle<Self::Message>
                + Handle<<S::SystemMessages as MessageParams>::Notification>
                + Handle<<S::SystemMessages as MessageParams>::Federated>;
        } else if #[cfg(notification)] {
            /// The type of the actor itself.
            type Actor: Actor
                + Handle<Self::Message>
                + Handle<<S::SystemMessages as MessageParams>::Notification>;
        } else if #[cfg(federated)] {
            /// The type of the actor itself.
            type Actor: Actor
                + Handle<Self::Message>
                + Handle<<S::SystemMessages as MessageParams>::Federated>;
        } else {
            /// The type of the actor itself.
            type Actor: Actor + Handle<Self::Message>;
        }
    }
}
