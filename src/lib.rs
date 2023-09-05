#![no_std]
#![cfg_attr(not(async_trait), feature(async_fn_in_trait))]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

extern crate alloc;

pub mod actor;

pub mod error;

pub mod message;

/// A utility used to store a two-way channel concisely
pub struct Channel<T>(pub flume::Sender<T>, pub flume::Receiver<T>);

impl<T> Channel<T> {
    /// Creates a new unbounded channel
    #[must_use]
    pub fn unbounded() -> Self {
        let c = flume::unbounded();
        Self(c.0, c.1)
    }
}
