//! # Fluxion Message
//! 
//! Contains the message trait used by Fluxion, as well as structures and utilities used by fluxion for message handling and serialization.
//! The [`Message`] trait should be implemented for all Messages that can be sent between actors, including Notifications and Federated Messages.
//! 
//! # Features
//! 
//! This crate includes features for `serde` support, and `federated` and `foreign` messages.
//! These features are enabled by fluxion as needed.

// The following will be in *every* crate related to fluxion.
#![no_std]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

extern crate alloc;

#[cfg(serde)]
use serde::{Deserialize, Serialize};

#[cfg(serde)]
pub mod serializer;

#[cfg(foreign)]
pub mod foreign;

pub mod handler;

/// # Message
/// This trait is used to mark Messages. Notifications are just Messages with a response type of `()`.
/// By default, all Messages and their responses must be [`Send`] + [`Sync`] + [`'static`].
/// When [`serde`] support is enabled, [`Serialize`] and [`Deserialize`] are also required.
#[cfg(not(serde))]
pub trait Message: Send + Sync + 'static {
    /// The message's response
    type Response: Send + Sync + 'static;
}
#[cfg(serde)]
pub trait Message: Serialize + for<'a> Deserialize<'a> + Send + Sync + 'static {
    /// The message's response
    type Response: Serialize + for<'a> Deserialize<'a> + Send + Sync + 'static;
}

impl Message for () {
    type Response = ();
}