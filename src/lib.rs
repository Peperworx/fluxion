//! # Fluxion
//! Distributed actor framework in Rust.
//!
//! Fluxion is an actor framework designed with distributed systems in mind, namely sending messages not just between actors, but also between systems.
//!
//! ## Messages and More
//! Fluxion implements three base methods of communication:
//!
//! ### Notifications
//! Notifications are messages that are broadcast across all actors in the system and connected systems.
//! Notifications do not accept a response, and have a unified datatype.
//!
//! ### Messages
//! Messages are sent directly between actors, and may or may not accept responses.
//! The datatype of the message is determined by the recieving actor.
//!
//! ### Federated Messages
//! Like Messages, Federated Message are sent directly between actors, and may or may not accept responses.
//! Unlike Messages, Federated Messages are "federated" in the sense that their datatype is the same across
//! the entire system, reguardless of the actor.
//!
//! ## Actors
//! Fluxion Actors are any struct, enum, or union that implements the [`Actor`] trait.
//! Actors must also implement the [`HandleFederated`] trait, and may choose to implement the
//! [`HandleNotification`] and [`HandleMessage`] traits any number of times. Be aware though, that special
//! circumstances determine which implemented handler will be used.

/// The implementation of actors and surrounding types.
pub mod actor;

/// The implementation of messages and surrounding types
pub mod message;

/// The implementation of systems and surrounding types
pub mod system;

/// Contains error types for the crate
pub mod error;

// If tracing is enabled, pub use the event macro
#[cfg(release_tracing)]
pub(crate) use tracing::event;

// If release tracing is not enabled and debug assertions are off,
// then we want to ignore Level::TRACE. So lets create a macro to do so.
#[cfg(all(tracing, not(release_tracing)))]
#[macro_export]
macro_rules! event {
    (Level::TRACE, $($_:tt)*) => {};
    ($($x:tt)*) => {
        tracing::event!($($x)*)
    };
}

// Otherwise define a dummy event macro.
#[cfg(not(tracing))]
#[macro_export]
macro_rules! event {
    ($($_:tt)*) => {};
}



pub use actor::{
    Actor, context::ActorContext,
    handle::ActorHandle,
    ActorID,
    supervisor::SupervisorErrorPolicy
};

pub use message::{
    Message, Notification,
    handler::{HandleFederated, HandleMessage, HandleNotification}
};

pub use system::System;
pub use error::{SystemError, ActorError, policy::{ErrorPolicy, ErrorPolicyCommand}};