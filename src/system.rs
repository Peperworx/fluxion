use std::{sync::Arc, collections::HashMap, marker::PhantomData};

use tokio::sync::{RwLock, broadcast};

use crate::actor::{ActorID, ActorType, Actor, ActorMessage, ActorSupervisor, ActorHandle};


/// An event that can be broadcast to all actors running on the system
pub trait SystemEvent: Clone + Send + Sync + 'static {}

/// The ID of a system. Currently defined as String
pub type SystemID = String;

/// The system that actors run on
#[derive(Clone)]
pub struct System<E: SystemEvent> {
    id: SystemID,
    actors: Arc<RwLock<HashMap<ActorID, Box<dyn ActorType>>>>,
    pub(crate) event_sender: broadcast::Sender<E>,
    _phantom_event: PhantomData<E>
}

impl<E: SystemEvent> System<E> {

    pub fn get_id(&self) -> SystemID {
        self.id.clone()
    }

    pub fn new(id: SystemID) -> Self {
        let (event_sender, _) = broadcast::channel(64);
        Self {
            id,
            actors: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            _phantom_event: PhantomData::default(),
        }
    }

    pub async fn add_actor<A: Actor<M, E>, M: ActorMessage>(&self, actor: A, id: ActorID) -> ActorHandle<M> {

        // Borrow actors as write
        let mut actors = self.actors.write().await;
        
        // If the key is already in actors, panic
        // Todo: Proper error handling
        if actors.contains_key(&id) {
            panic!("Actor already exists");
        }

        // Clone the system (self)
        let system = self.clone();
        
        // Create the runner
        let (mut runner, handle) = ActorSupervisor::new(actor, id.clone(), system.clone());

        // Start the supervisor task
        tokio::spawn(async move {
            runner.run(system).await;
        });
        
        // Box the actor handle
        let boxed_handle = Box::new(handle.clone());

        // Insert it into the map
        actors.insert(id, boxed_handle);

        // Return the handle
        handle
    }
}