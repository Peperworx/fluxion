//! Contains [`Fluxion`], the main implementor of [`System`].

#[cfg(serde)]
use serde::{Deserialize, Serialize};

#[cfg(foreign)]
use crate::types::errors::ForeignError;

use crate::message::{foreign::ForeignMessage, foreign::ForeignHandle};

use core::marker::PhantomData;

use alloc::{collections::BTreeMap, sync::Arc, boxed::Box, vec::Vec};
use maitake_sync::RwLock;

use crate::{actor::{handle::{ActorHandle, LocalHandle}, supervisor::Supervisor}, FluxionParams, Actor, Executor, Handler, Message, ActorId, MessageSender, System, ActorContext};


/// The type alias for the map of actors stored in the system.
type ActorMap = BTreeMap<Arc<str>, Box<dyn ActorHandle>>;

#[cfg(foreign)]
type ForeignMap = BTreeMap<Arc<str>, whisk::Channel<Option<ForeignMessage>>>;

/// # [`Fluxion`]
/// The core management functionality of fluxion.
/// Handles the creation, shutdown, and management of actors.
/// 
/// Used `Arc` internally, so this can be cloned around, although it is not recommended.
/// Immutable references can be used to do most things instead.
#[derive(Clone)]
pub struct Fluxion<C: FluxionParams> {
    /// The map of actors
    actors: Arc<RwLock<ActorMap>>,
    /// The system's ID
    id: Arc<str>,
    /// Inbound foreign message channels
    #[cfg(foreign)]
    foreign: Arc<RwLock<ForeignMap>>,
    /// Outbound foreign message channel
    #[cfg(foreign)]
    outbound_foreign: whisk::Channel<ForeignMessage>,
    /// Phantom data associating the generics with this struct
    _phantom: PhantomData<C>,
}

impl<C: FluxionParams> Fluxion<C> {
    /// Creates a new [`Fluxion`] instance
    #[must_use]
    pub fn new(id: &str) -> Self {
        Fluxion {
            actors: Arc::new(RwLock::new(BTreeMap::default())),
            id: id.into(),
            #[cfg(foreign)]
            foreign: Arc::new(RwLock::new(BTreeMap::default())),
            #[cfg(foreign)]
            outbound_foreign: whisk::Channel::new(),
            _phantom: PhantomData,
        }
    }

    /// Gets the outbound foreign channel
    #[cfg(foreign)]
    #[must_use]
    pub fn outbound_foreign(&self) -> whisk::Channel<ForeignMessage> {
        self.outbound_foreign.clone()
    }

    /// Gets the system's id
    /// 
    /// # Returns
    /// Returns the local system's id.
    #[must_use]
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Shutdown and remove all local actors.
    /// 
    /// # Returns
    /// Returns the number of actors shutdown
    pub async fn shutdown(&self) -> usize {
        // Lock actors as write
        let mut actors = self.actors.write().await;

        // Create a list of shutdown_receivers
        let mut shutdown_receivers = Vec::new();

        // Begin the shutdown process of each actor
        while let Some((_, actor)) = actors.pop_first() {
            // Shutdown the actor and push to shutdown_receivers
            if let Some(receiver) = actor.begin_shutdown().await {
                shutdown_receivers.push(receiver);
            }
        }

        // Count the number of shutdown actors
        let shutdown_actors = shutdown_receivers.len();

        // Wait for every actor to shutdown
        for rcv in shutdown_receivers {
            // We don't care about any errors.
            let _ = rcv.await;
        }

        // Return the number of shutdown actors.
        shutdown_actors
    }

    /// Relays a foreign message to an actor on this system.
    /// Upon error, the caller should respond to the message with `None`,
    /// unless there is a possibility of recovery.
    /// 
    /// # Errors
    /// Returns an error if the foreign message failed to relay to this system.
    #[cfg(foreign)]
    pub async fn relay_foreign(&self, message: ForeignMessage) -> Result<(), ForeignError> {

        // If for some reason the message's system does not match, error
        if message.target.get_system() != self.id.as_ref() {
            return Err(ForeignError::SystemNoMatch);
        }

        // Lock the foreign channel
        let foreign = self.foreign.read().await;

        // Try to lookup the foregin channel for the actor
        let Some(fc) = foreign.get(message.target.get_actor()) else {
            return Err(ForeignError::NoActor);
        };

        // Relay the message
        fc.send(Some(message)).await;

        // OK
        Ok(())
    }
    
    /// Internal implementation of [`System::add`]
    pub(crate) async fn add_internal<A: Actor<C>>(&self, actor: A, id: &str, owner: Option<ActorId>) -> Option<LocalHandle<C, A>> {

        // Create the actor id
        let aid = ActorId::from(id);

        // Strip the system, and insert this system in
        let id = alloc::string::String::from(self.id.as_ref());
        let id = ActorId::from(id + ":" + aid.get_actor());

        // If the actor already exists, then return None.
        // Lock actors as read here temporarily.
        if self.actors.read().await.contains_key(id.get_actor()) {
            return None;
        }

        // Create the actor's context
        let context = ActorContext::new(id.clone(), self.clone());

        // Create the supervisor
        let mut supervisor = Supervisor::<C, A>::new(actor, context);

        // Get a handle.
        let handle = supervisor.handle(owner);

        // Start a task for the supervisor
        <C::Executor as Executor>::spawn(async move {
            // Run the supervisor
            if supervisor.run().await.is_err() {
                todo!("Error handling");
            }
        });

        // Lock the actors map as write
        let mut actors = self.actors.write().await;
        
        // Insert a clone of the handle in the actors list
        actors.insert(id.get_actor().into(), Box::new(handle.clone()));

        // Return the handle
        Some(handle)
    }

    /// Internal implementation of [`System::get_local`]
    pub(crate) async fn get_local_internal<A: Actor<C>>(&self, id: &str, owner: Option<ActorId>) -> Option<LocalHandle<C, A>> {
        // Lock the map as read
        let actors = self.actors.read().await;

        // Get the actor, and map the value to a downcast
        // Additionally, update the owner of the handle.
        actors.get(id)
            .and_then(|v| v.as_any().downcast_ref().cloned())
            .map(|mut v: LocalHandle<C, A>| {
                v.owner = owner;
                v
            })
    }

    /// # Internal implementation of [`System::get`]
    /// Get an actor from its id as a `Box<dyn MessageSender>`.
    /// Use this for most cases, as it will also handle foreign actors.
    /// Takes an additional parameter for the owner of the returned handle.
    #[cfg(foreign)]
    pub(crate) async fn get_internal<
        A: Handler<C, M>,
        M: Message + Serialize>(&self, id: ActorId, owner: Option<ActorId>) -> Option<Box<dyn MessageSender<M>>>
    where
        M::Response: for<'a> Deserialize<'a>
    {
        
        // If the system is the local system, find the actor
        if id.get_system() == self.id.as_ref() || id.get_system().is_empty() {
            
            // Lock actors as read
            let actors = self.actors.read().await;
            
            // Get the actor, returning None if it does not exist
            let actor = actors.get(id.get_actor())?;
            
            // Try to downcast to a concrete type
            let actor: &LocalHandle<C, A> = actor.as_any().downcast_ref().as_ref()?;

            // Clone and box the handle
            let mut handle = Box::new(actor.clone());

            // Update the owner
            handle.owner = owner;

            // Return it
            Some(handle)
        } else {
            
            // Get an owner for the message. If no owner, just use the current system
            let owner = owner.unwrap_or_else(|| {
                let v = alloc::string::String::from(self.id.as_ref()) + ":";
                v.into()
            });

            // Create a foreign message handle
            let foreign_handle = ForeignHandle::<C::Serializer>::new(self.outbound_foreign.clone(), id, owner);

            // Box it and return
            Some(Box::new(foreign_handle))
        }
    }

    #[cfg(not(foreign))]
    pub(crate) async fn get_internal<
        A: Handler<C, M>,M: Message,
        #[cfg(foreign)] M: Message + Serialize>(&self, id: ActorId, owner: Option<ActorId>) -> Option<Box<dyn MessageSender<M>>> {
        
        // If the system is the local system, find the actor
        if id.get_system() == self.id.as_ref() || id.get_system().is_empty() {
            
            // Lock actors as read
            let actors = self.actors.read().await;
            
            // Get the actor, returning None if it does not exist
            let actor = actors.get(id.get_actor())?;
            
            // Try to downcast to a concrete type
            let actor: &LocalHandle<C, A> = actor.as_any().downcast_ref().as_ref()?;

            // Clone and box the handle
            let mut handle = Box::new(actor.clone());

            // Update the owner
            handle.owner = owner;

            // Return it
            Some(handle)
        } else  {
            // If foreign messages are disabled, return None
            None
        }
    }
}


#[cfg_attr(async_trait, async_trait::async_trait)]
impl<C: FluxionParams> System<C> for Fluxion<C> {
    /// Add an actor to the system
    /// 
    /// # Returns
    /// Returns [`None`] if the actor was not added to the system.
    /// If the actor was added to the system, returns [`Some`]
    /// containing the actor's [`LocalHandle`].
    async fn add<A: Actor<C>>(&self, actor: A, id: &str) -> Option<LocalHandle<C, A>> {
        // Use the internal add function, with No default owner
        self.add_internal(actor, id, None).await
    }

    
    #[cfg(foreign)]
    async fn foreign_proxy<A, M, R>(&self, actor_id: &str, foreign_id: &str) -> bool
    where
        A: Handler<C, M>,
        M: Message<Response = R> + Serialize + for<'a> Deserialize<'a>,
        R: Send + Sync + 'static + Serialize + for<'a> Deserialize<'a>,
    {
        
        // Get the actor as a local handle, returning false
        // if it can not be found.

        use crate::{types::serialize::MessageSerializer, Event};
        let Some(actor) = self.get_local::<A>(actor_id).await else {
            return false;
        };

        // Lock the foreign handler map
        let mut foreign = self.foreign.write().await;
        
        // If the foreign id already exists, return false
        if foreign.contains_key(foreign_id) {
            return false;
        }

        // Create a new channel for foreign messages to this actor
        let channel = whisk::Channel::<Option<ForeignMessage>>::new();

        // Clone it for the new task
        let rx = channel.clone();

        // Spawn a new task that receives, deserializes, and dispatches.
        <C::Executor as Executor>::spawn(async move {
            loop {
                // Receive on the channel
                let next = rx.recv().await;

                // If None, exit
                let Some(mut next) = next else {
                    break;
                };

                // Deserialize, continuing if it fails
                let Some(message) = <C::Serializer as MessageSerializer>::deserialize::<M>(next.message) else {
                    continue;
                };

                // Dispatch the message
                let res = actor.request_internal(Event {
                    message,
                    source: Some(next.source),
                    target: next.target
                }).await;

                // Serialize the response, continue if it fails.
                let Some(res) = <C::Serializer as MessageSerializer>::serialize(res) else {
                    continue;
                };

                // Send the response
                let _ = next.responder.send(Some(res));
            }
        });

        // Now that foreign messages are being handled, add the channel
        foreign.insert(foreign_id.into(), channel);

        true
    }

    

    /// Get a local actor as a `LocalHandle`. Useful for running management functions like shutdown
    /// on known local actors.
    async fn get_local<A: Actor<C>>(&self, id: &str) -> Option<LocalHandle<C, A>> {
        // Delegate to internal implementation, using no owner
        self.get_local_internal(id, None).await
    }

    /// Get an actor from its id as a `Box<dyn MessageSender>`.
    /// Use this for most cases, as it will also handle foreign actors.
    #[cfg(foreign)]
    async fn get<
        A: Handler<C, M>,
        M: Message + Serialize>(&self, id: ActorId) -> Option<Box<dyn MessageSender<M>>>
    where
        M::Response: for<'a> Deserialize<'a>
    {
        self.get_internal::<A,M>(id, None).await
    }
        
    #[cfg(not(foreign))]
    async fn get<
        A: Handler<C, M>,
        M: Message>(&self, id: ActorId) -> Option<Box<dyn MessageSender<M>>>
    {
        self.get_internal::<A,M>(id, None).await
    }
    

    /// Removes an actor from the system, and waits for it to stop execution
    async fn remove(&self, id: &str) {
        // Remove the actor and get it's original handle
        let actor = self.actors.write().await.remove(id);

        // If it did not exist, we don't care.
        // But if it did, shut it down.
        if let Some(actor) = actor {
            // Begin the shutdown
            let rx = actor.begin_shutdown().await;

            // And if a receiver was provided, wait for it to complete
            let Some(rx) = rx else {
                return;
            };

            // We don't care if this does not run.
            // If it doesn't, the actor has just shutdown.
            let _ = rx.await;
        }
    }
}