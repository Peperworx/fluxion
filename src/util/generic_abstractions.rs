//! # Generic Abstractions
//! This file contains a way to abstract many different generics and feature flags into a singular, concise generic
//! that uses associated types to store the values.
//! This method significantly reduces repetition and makes code much easier to read.
//!
//! ## [`SystemParams`]
//! This is the basic generic abstraction. It does not depend on any other generic abstractions, and it contains
//! only two associated types: `Federated` and `Notification`, which only exist if their respective feature flags are enabled.

use crate::message::Message;

/// # [`SystemParams`]
/// The simplest generic abstraction, containing the federated message and notification types.
pub trait SystemParams {
    /// The federated message associated with the system
    #[cfg(federated)]
    type Federated: Message;
}
