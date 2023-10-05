//! Provides a [`System`], which stores and manages many different actors

use core::{any::Any, marker::PhantomData};

use alloc::{collections::BTreeMap, boxed::Box, sync::Arc};
use maitake_sync::RwLock;

use crate::{types::{actor::{ActorId, Actor}, params::{SystemParams, SupervisorGenerics}, executor::Executor, message::{MessageSender, Message}, Handle}, supervisor::Supervisor, handle::{ActorHandle, LocalHandle}};



pub struct System<Params: SystemParams> {
    /// The map of actors stored in the system
    actors: Arc<RwLock<BTreeMap<ActorId, Box<dyn ActorHandle>>>>,
    /// The system's ID
    id: Arc<str>,
    /// The underlying executor
    executor: Params::Executor,
}

impl<Params: SystemParams> System<Params> {

    /// Creates a new system
    #[must_use]
    pub fn new(id: &str, executor: Params::Executor) -> Self {
        Self {
            actors: Arc::new(RwLock::new(BTreeMap::default())),
            id: Arc::from(id),
            executor
        }
    }


    /// Adds an actor to the system, and returns a handle
    pub async fn add<A: Actor>(&self, actor: A, id: ActorId) -> Option<LocalHandle<A>> {

        // If the actor already exists, then return None.
        // Lock actors as read here temporarily.
        if self.actors.read().await.contains_key(&id) {
            return None;
        }

        // Create the supervisor
        let supervisor = Supervisor::<SupervisorGenerics<A>>::new(actor);

        // Get a handle
        let handle = supervisor.handle();

        // Start a task for the supervisor
        self.executor.spawn(async move {
            loop {
                // Tick the supervisor
                let res = supervisor.tick().await;

                // TODO: tracing here
                if res.is_err() {
                    continue;
                }
            }
        });

        // Lock the actors map as write
        let mut actors = self.actors.write().await;
        
        // Insert a clone of the handle in the actors list
        actors.insert(id, Box::new(handle.clone()));

        // Return the handle
        Some(handle)
    }

    /// Retrieves a [`MessageSender`] for a given actor id and message. This will be a [`LocalHandle`]
    /// if the actor is on the current system, but will be a [`ForeignHandle`] for foreign actors. [`None`] will
    /// be returned if the target is the local system, but the actor is not found. Actors on foreign systems will
    /// always be returned, as long as foreign messages are enabled. If they are not, then None will be returned
    /// for all foreign actors.
    pub async fn get<A: Handle<M>, M: Message>(&self, id: ActorId) -> Option<Box<dyn MessageSender<M>>> {

        // If the system is the local system, find the actor
        if id.get_system() == self.id.as_ref() || id.get_system() == "" {
            
            // Get the actor, returning None if it does not exist
            let actor = self.actors.read().await.get(&id)?;

            // Try to downcast to a concrete type
            let actor: &LocalHandle<A> = actor.as_any().downcast_ref().as_ref()?;

            // Clone and box the handle
            let handle = Box::new(actor.clone());

            // Return it
            Some(handle)
        } else {#[cfg(not(foreign))] {
            // If foreign messages are disabled, return None
            None
        } #[cfg(foreign)] {


            todo!()
        }}
    }
}