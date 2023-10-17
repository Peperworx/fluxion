//! Contains structures that allow an actor to access its outside world.


use crate::ActorId;


/// # [`Context`]
/// This trait allows an actor's context to be defined generically, and provides functions for interacting with
/// the system and retrieving details about the actor.

pub trait Context: Send + Sync + 'static {
    /// Retrieve's the actor's ID
    fn get_id(&self) -> ActorId;
}

/// # [`ActorContext`]
/// Implements [`Context`] and [`System`].
pub struct ActorContext {

    /// The actor's ID
    id: ActorId,
}

impl ActorContext {
    #[must_use]
    pub fn new(id: ActorId) -> Self {
        Self {
            id
        }
    }
}



impl Context for ActorContext {
    fn get_id(&self) -> ActorId {
        self.id.clone()
    }
}