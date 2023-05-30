use std::{sync::Arc, collections::HashMap, marker::PhantomData};

use tokio::sync::{RwLock, broadcast};

use crate::actor::{ActorID, ActorType, Actor, ActorMessage, ActorSupervisor, ActorHandle};


/// An event that can be broadcast to all actors running on the system
pub trait SystemEvent: std::fmt::Debug + Clone + Send + Sync + 'static {}

/// The ID of a system. Currently defined as String
pub type SystemID = String;

/// The system that actors run on
#[derive(Clone)]
pub struct System<E: SystemEvent> {
    id: SystemID,
    actors: Arc<RwLock<HashMap<ActorID, Box<ActorType>>>>,
    pub(crate) shutdown_sender: broadcast::Sender<()>,
    pub(crate) event_sender: broadcast::Sender<E>,
    _phantom_event: PhantomData<E>
}

impl<E: SystemEvent> System<E> {

    pub fn get_id(&self) -> SystemID {
        self.id.clone()
    }

    pub fn new(id: SystemID) -> Self {
        let (event_sender, _) = broadcast::channel(64);
        let (shutdown_sender, _) = broadcast::channel(16);
        Self {
            id,
            actors: Default::default(),
            shutdown_sender,
            event_sender,
            _phantom_event: PhantomData::default(),
        }
    }

    pub async fn send_event(&self, event: E) {
        // Todo: Proper error handling.
        self.event_sender.send(event).unwrap();
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

    pub async fn get_actor<A: Actor<M, E>, M: ActorMessage>(&self, id: &ActorID) -> Option<ActorHandle<M>> {

        // Lock read access to actors
        let actors = self.actors.read().await;

        // Try to get the actor
        let actor = actors.get(id);
        
        // If the actor is Some, downcast it to ActorRef<M>. If it fails, return None
        actor.and_then(|boxed| {
            boxed.downcast_ref().cloned()
        })
    }

    pub async fn remove_actor(&self, id: &ActorID) {
        
        // Lock write access to actors
        let mut actors = self.actors.write().await;

        // Remove the actor. This will kill the actor by removing the only reference
        // to its message sender that is guarenteed to always exist for the actor's lifetime.
        // Once all remaining ActorHandles are dropped, the actor's main loop will terminate,
        // call the deinitialize function, and cleanup the message recievers.
        actors.remove(id);
    }

    pub async fn shutdown(&self) {
        // Lock actors for write access
        let mut actors = self.actors.write().await;
        
        // Clear every actor
        actors.clear();

        // Drop the actors handle
        drop(actors);

        // Send the shutdown broadcast
        if let Err(_) = self.shutdown_sender.send(()) {
            // If there is an error here, there are no active supervisors so we can return
            return;
        }

        // Wait until there are no more recievers listining
        while self.shutdown_sender.receiver_count() > 0 {
            // Yield execution back to other tasks so that they can finish
            tokio::task::yield_now().await;
        }
    }
}