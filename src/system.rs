use std::{sync::Arc, collections::HashMap, any::Any};

use tokio::sync::RwLock;

use crate::actor::{ActorID, ActorType};


/// An event that can be broadcast to all actors running on the system
pub trait SystemEvent: Clone + Send + Sync + 'static {}

/// The ID of a system. Currently defined as String
pub type SystemID = String;

/// The system that actors run on
#[derive(Clone)]
pub struct System {
    id: SystemID,
    actors: Arc<RwLock<HashMap<ActorID, Box<dyn ActorType>>>>,

}

impl System {

    
}