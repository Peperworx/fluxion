//! This module contains many utilities that significantly simplify fluxion's usage.
//! Many of these are `pub use`d from the crate root.

use crate::message::Message;

pub mod params;

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

/// [`Message`] is implemented for [`()`].
/// This can be used for testing purposes, and is done because other crates can't implement [`Message`] on [`()`] anyways.
impl Message for () {
    type Response = ();

    type Error = ();
}
