//! # `System`
//! This module contains the system that handles all running actors.

use alloc::{boxed::Box, collections::BTreeMap, string::String};

use crate::message::Dispatcher;

/// # `System`
/// The system contains all actors, and is responsible for dispatching messages to other actors.
/// It is responsible for ticking all ActorSupervisors, which allow the user to chose an async executor.
///
pub struct System {
    actors: BTreeMap<String, Box<dyn Dispatcher>>,
}

impl System {
    /// Adds an actor with a given ID to the system.
    pub fn add_actor(&mut self) {
        todo!()
    }
}
