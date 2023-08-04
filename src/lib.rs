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

pub use actor::{
    Actor,
    handle::ActorHandle,
    ActorID,
    supervisor::SupervisorErrorPolicy
};

#[cfg(not(feature = "foreign"))]
use crate::message::DefaultFederated;

#[cfg(not(feature="notifications"))]
use crate::message::DefaultNotification;

/// # ActorContext
/// [`ActorContext`] provides methods to allow an actor to interact with its [`System`] and other actors.
/// This is done instead of providing a [`System`] reference directly to disallow actors from calling notifications and calling into themselves, which
/// can cause infinite loops.
#[cfg(all(feature = "notifications", feature = "foreign"))]
pub type ActorContext<F, N> = actor::context::ActorContext<F, N>;
#[cfg(all(feature = "notifications", not(feature = "foreign")))]
pub type ActorContext<N> = actor::context::ActorContext<DefaultFederated, N>;
#[cfg(all(not(feature = "notifications"), feature = "foreign"))]
pub type ActorContext<F> = actor::context::ActorContext<F, DefaultNotification>;
#[cfg(all(not(feature = "notifications"), not(feature = "foreign")))]
pub type ActorContext = actor::context::ActorContext<DefaultFederated, DefaultNotification>;

pub use message::{
    Message, Notification,
    handler::{HandleFederated, HandleMessage, HandleNotification}
};

pub use system::System;
pub use error::{SystemError, ActorError, policy::{ErrorPolicy, ErrorPolicyCommand}};