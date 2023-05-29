use std::{sync::Arc, collections::HashMap};

use tokio::sync::RwLock;

use crate::actor::{ActorID, Actor};



/// The system that actors run on
pub struct System {
    actors: Arc<RwLock<HashMap<ActorID, Box<dyn Actor>>>>,
}