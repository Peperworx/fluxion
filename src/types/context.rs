//! Contains structures that allow an actor to access its outside world.

use crate::system::System;
use alloc::boxed::Box;

/// # [`ActorContext`]
/// This struct allows an actor to interact with other actors
pub struct ActorContext(Box<dyn System>);