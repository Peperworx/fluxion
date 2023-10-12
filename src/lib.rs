#! [doc = include_str! ("../README.md")]


#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(async_trait), feature(async_fn_in_trait))]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]



extern crate alloc;


pub mod types;

pub mod supervisor;

pub mod handle;

pub mod system;


use alloc::{collections::BTreeMap, sync::Arc, boxed::Box};
use handle::{ActorHandle, LocalHandle};
use maitake_sync::RwLock;
use supervisor::Supervisor;
use system::System;
use types::context::ActorContext;
use types::executor::Executor;
use types::{params::{FluxionParams, SupervisorGenerics}, broadcast, actor::{Actor, ActorId}, Handle, message::{Message, MessageSender}};

type ActorMap = BTreeMap<Arc<str>, Box<dyn ActorHandle>>;

/// # [`Fluxion`]
/// [`Fluxion`] holds a map of actor references, and manages the creation and retreval of actors, as well as their lifeitmes.
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
    pub fn new(id: &str, executor: Params::Executor) -> Self {

        Self {
            actors: Arc::new(RwLock::new(BTreeMap::new())),
            id: id.into(),
            executor: Arc::new(executor),
            shutdown: broadcast::channel(64),
        }
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<Params: FluxionParams> System for Fluxion<Params> {

    type Context = ActorContext<Params>;
    
    async fn add<A: Actor<Context = Self::Context>>(&self, actor: A, id: &str) -> Option<LocalHandle<A>> {

        // If the actor already exists, then return None.
        // Lock actors as read here temporarily.
        if self.actors.read().await.contains_key(id) {
            return None;
        }

        // Create the actor's context
        let context = ActorContext::new(
            self.clone(), id.into()
        );

        // Create the supervisor
        let supervisor = Supervisor::<SupervisorGenerics<A>>::new(actor, context, self.shutdown.subscribe().await);

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
        actors.insert(id.into(), Box::new(handle.clone()));

        // Return the handle
        Some(handle)
    }
    
    
    async fn get_local<A: Actor<Context = Self::Context>>(&self, id: &str) -> Option<LocalHandle<A>> {
        // Lock the map as read
        let actors = self.actors.read().await;

        // Get the actor, and map the value to a downcast
        actors.get(id)
            .and_then(|v| v.as_any().downcast_ref().cloned())
    }
    
    
    async fn get<A: Handle<M, Context = Self::Context>, M: Message>(&self, id: ActorId) -> Option<Box<dyn MessageSender<M>>> {
        
        // If the system is the local system, find the actor
        if id.get_system() == self.id.as_ref() || id.get_system().is_empty() {
            
            // Lock actors as read
            let actors = self.actors.read().await;
            
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

    async fn remove(&self, id: &str) {
        self.actors.write().await.remove(id);
    }
}
