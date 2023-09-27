//! # Actor Supervisor
//! This module contains the [`Supervisor`]. This struct contains an actor, alongside code dedicated to handling messages for the actor.

use alloc::{sync::Arc, boxed::Box};

use crate::types::{params::SupervisorParams, message::{MessageHandler, Message}, errors::ActorError, actor::Actor};


/// # [`Supervisor`]
/// This struct wraps an actor, and is owned by a task which constantly receives messages over an asynchronous mpsc channel.
pub struct Supervisor<A: Actor> {
    /// The supervised actor
    actor: A,
}

impl<A: Actor> Supervisor<A> {

}