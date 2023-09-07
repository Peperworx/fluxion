//! Contains the [`ActorEntry`] trait, which is stored in the [`crate::System`].

use core::any::Any;

use crate::{error::FluxionError, message::foreign::ForeignMessage};

#[cfg(async_trait)]
use alloc::boxed::Box;

/// # [`ActorEntry`]
/// Implemented on [`ActorRef`], allowing it to be stored in the system
/// while simultaneously erasing its message type and allowing foreign messages to be sent to it.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait ActorEntry: Any {
    /// Returns the implementing type as an &dyn Any, allowing it to then be downcast
    /// to the actual [`ActorRef`] for local usage.
    fn as_any(&self) -> &dyn Any;

    /// Handle a foreign message on this [`ActorRef`]
    ///
    /// # Errors
    /// This function may error whenever handling the foreign message fails.
    /// This varies depending on implementation.
    async fn handle_foreign<E>(&self, message: ForeignMessage) -> Result<(), FluxionError<E>>;
}
