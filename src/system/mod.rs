//! # System
//! The system forms the core of Fluxion. It is responsible for the storage, creation, and management of actors.

use alloc::{collections::BTreeMap, string::String, sync::Arc};
use async_rwlock::RwLock;

pub struct System {
    /// The map which contains every actor.
    /// This is wrapped in an [`Arc`] and [`RwLock`] to allow it to be accessed from many different tasks.
    ///
    actors: Arc<RwLock<BTreeMap<String, ()>>>,
}

impl System {
    pub fn new() -> Self {
        Self {
            actors: Default::default(),
        }
    }

    /// Adds a new actor to the system and begins running the supervisor
    pub fn add<A>(&self, id: String, actor: A) {
        todo!()
    }
}
