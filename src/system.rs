//! Provides a [`System`], which stores and manages many different actors



use alloc::{collections::BTreeMap, boxed::Box, sync::Arc};
use maitake_sync::RwLock;

use crate::{types::{actor::{ActorId, Actor}, params::{FluxionParams, SupervisorGenerics}, executor::Executor, message::{MessageSender, Message}, Handle, broadcast}, supervisor::Supervisor, handle::{ActorHandle, LocalHandle}, ActorMap};

/// # [`System`] 
/// [`System`] is an internal trait that allows access to the utilities provided by [`Fluxion`] without requiring any generics.
/// The trait [`Manager`] is implemented for `&System`, and provides more advanced features at the cost of not being object safe.
pub(crate) trait System: Send + Sync + 'static {
    /// The system's parameters
    type Params: FluxionParams;

    /// Retreives an Arc clone of the underlying actors list
    fn get_actors(&self) -> Arc<RwLock<ActorMap>>;

    /// Get the system's id
    fn get_id(&self) -> Arc<str>;

    /// Gets the shutdown channel
    fn get_shutdown(&self) -> broadcast::Sender<()>;
}

#[cfg_attr(async_trait, async_trait::async_trait)]
pub(crate) trait Manager {

    /// Adds an actor to the system, and returns a handle
    async fn add<A: Actor>(&self, actor: A, id: &str) -> Option<LocalHandle<A>>;

    /// Retrieves a [`LocalHandle`], which allows every message type an actor supports to be used.
    /// This will return None if the actor does not exist.
    async fn get_local<A: Actor>(&self, id: &str) -> Option<LocalHandle<A>>;

    /// Retrieves a [`MessageSender`] for a given actor id and message. This will be a [`LocalHandle`]
    /// if the actor is on the current system, but will be a [`ForeignHandle`] for foreign actors. [`None`] will
    /// be returned if the target is the local system, but the actor is not found. Actors on foreign systems will
    /// always be returned, as long as foreign messages are enabled. If they are not, then None will be returned
    /// for all foreign actors.
    async fn get<A: Handle<M>, M: Message>(&self, id: ActorId) -> Option<Box<dyn MessageSender<M>>>;
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<Params: FluxionParams> Manager for &(dyn System<Params = Params> + 'static) {
    
    async fn add<A: Actor>(&self, actor: A, id: &str) -> Option<LocalHandle<A>> {

        // If the actor already exists, then return None.
        // Lock actors as read here temporarily.
        if self.get_actors().read().await.contains_key(id) {
            return None;
        }

        // Create the supervisor
        let supervisor = Supervisor::<SupervisorGenerics<A>>::new(actor, self.get_shutdown().subscribe().await);

        // Get a handle
        let handle = supervisor.handle();

        // Start a task for the supervisor
        Params::Executor::spawn(async move {
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
        let actors = self.get_actors();
        let mut actors = actors.write().await;
        
        // Insert a clone of the handle in the actors list
        actors.insert(id.into(), Box::new(handle.clone()));

        // Return the handle
        Some(handle)
    }
    
    
    async fn get_local<A: Actor>(&self, id: &str) -> Option<LocalHandle<A>> {
        // Lock the map as read
        let actors = self.get_actors();
        let actors = actors.read().await;

        // Get the actor, and map the value to a downcast
        actors.get(id)
            .and_then(|v| v.as_any().downcast_ref().cloned())
    }
    
    
    async fn get<A: Handle<M>, M: Message>(&self, id: ActorId) -> Option<Box<dyn MessageSender<M>>> {
        
        // If the system is the local system, find the actor
        if id.get_system() == self.get_id().as_ref() || id.get_system().is_empty() {
            
            // Lock actors as read
            let actors = self.get_actors();
            let actors = actors.read().await;
            
            // Get the actor, returning None if it does not exist
            let actor = actors.get(id.get_actor())?;
            
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

/// # [`System`]
/// The [`System`] holds a map of actor references, and manages the creation and retreval of actors, as well as their lifeitmes.
pub struct Fluxion<Params: FluxionParams> {
    /// The map of actors stored in the system
    pub(crate) actors: Arc<RwLock<ActorMap>>,
    /// The system's ID
    pub(crate) id: Arc<str>,
    /// The underlying executor
    pub(crate) executor: Arc<Params::Executor>,
    /// The shutdown message sender
    pub(crate) shutdown: broadcast::Sender<()>,
}

impl<Params: FluxionParams> Clone for Fluxion<Params> {
    fn clone(&self) -> Self {
        Self { actors: self.actors.clone(), id: self.id.clone(), executor: self.executor.clone(), shutdown: self.shutdown.clone() }
    }
}

impl<Params: FluxionParams> Fluxion<Params> {

    /// Creates a new system
    #[must_use]
    pub fn new(id: &str, executor: Params::Executor) -> Self {
        Self {
            actors: Arc::new(RwLock::new(BTreeMap::default())),
            id: Arc::from(id),
            executor: Arc::new(executor),
            shutdown: broadcast::channel(1)
        }
    }

    /// Adds an actor to the system, and returns a handle
    pub async fn add<A: Actor>(&self, actor: A, id: &str) -> Option<LocalHandle<A>> {
        let s: &dyn System<Params = Params> = self;

        s.add(actor, id).await
    }

    /// Retrieves a [`LocalHandle`], which allows every message type an actor supports to be used.
    /// This will return None if the actor does not exist.
    pub async fn get_local<A: Actor>(&self, id: &str) -> Option<LocalHandle<A>> {
        let s: &dyn System<Params = Params> = self;
        s.get_local(id).await
    }

    /// Retrieves a [`MessageSender`] for a given actor id and message. This will be a [`LocalHandle`]
    /// if the actor is on the current system, but will be a [`ForeignHandle`] for foreign actors. [`None`] will
    /// be returned if the target is the local system, but the actor is not found. Actors on foreign systems will
    /// always be returned, as long as foreign messages are enabled. If they are not, then None will be returned
    /// for all foreign actors.
    pub async fn get<A: Handle<M>, M: Message>(&self, id: ActorId) -> Option<Box<dyn MessageSender<M>>> {
        let s: &dyn System<Params = Params> = self;
        s.get::<A, M>(id).await
    }
}

impl<Params: FluxionParams> System for Fluxion<Params> {
    type Params = Params;

    fn get_actors(&self) -> Arc<RwLock<ActorMap>> {
        self.actors.clone()
    }

    fn get_id(&self) -> Arc<str> {
        self.id.clone()
    }

    fn get_shutdown(&self) -> broadcast::Sender<()> {
        self.shutdown.clone()
    }
}