//! The implementation of systems and surrounding types

use std::{collections::HashMap, sync::Arc};


use tokio::sync::{broadcast, RwLock};

#[cfg(feature = "foreign")]
use tokio::sync::{mpsc, Mutex};

#[cfg(feature = "tracing")]
use tracing::{event, Level};

use crate::{
    actor::{
        handle::local::LocalHandle,
        supervisor::{ActorSupervisor, SupervisorErrorPolicy},
        Actor, ActorEntry,
    },
    error::SystemError,
    message::{
        handler::{HandleFederated, HandleMessage, HandleNotification},
        Message, Notification
    },
};

#[cfg(feature="foreign")]
use crate::{
    actor::{
        ActorID,
        handle::{
            foreign::ForeignHandle,
            ActorHandle
        }
    },
    error::ActorError
};

#[cfg(feature="foreign")]
use crate::message::foreign::ForeignMessage;

#[cfg(any(not(feature="foreign"), not(feature="notifications")))]
use std::marker::PhantomData;


/// Internals used when foreign messages are enabled
#[cfg(feature = "foreign")]
mod foreign;

/// # ActorType
/// The type of an actor in the hashmap
#[cfg(feature = "foreign")]
type ActorType<F> = Box<dyn ActorEntry<Federated = F> + Send + Sync + 'static>;

#[cfg(not(feature = "foreign"))]
type ActorType<F> = (Box<dyn ActorEntry + Send + Sync + 'static>, PhantomData<F>);

/// # GetActorReturn
/// The return value of the `get_actor` function
#[cfg(feature = "foreign")]
pub(crate) type GetActorReturn<F, M> = Box<dyn ActorHandle<F, M>>;

#[cfg(not(feature = "foreign"))]
pub(crate) type GetActorReturn<F, M> = LocalHandle<F, M>;

/// # System
/// The core part of Fluxion, the [`System`] runs actors and handles communications between other systems.
///
/// ## Inter-System Communication
/// Fluxion systems enable communication by having what is called a foreign channel.
/// The foreign channel is an mpsc channel, the Reciever for which can be retrieved once by a single outside source using [`System::get_foreign`].
/// When a Message or Foreign Message is sent to an external actor, or a Notification is sent at all, the foreign
/// channel will be notified.
/// 
/// ## Using Clone
/// System uses [`Arc`] internally, so a [`System`] can be cloned where needed.
#[derive(Clone)]
pub struct System<F, N>
where
    F: Message,
{
    /// The id of the system
    id: String,

    /// Internals used when foreign messages are enabled
    #[cfg(feature = "foreign")]
    foreign: foreign::ForeignComponents<F, N>,

    /// Foreign messages are not enabled, the system needs a PhantomData to hold F
    #[cfg(not(feature = "foreign"))]
    _phantom: PhantomData<F>,

    /// A shutdown sender which tells all actors to stop
    shutdown: broadcast::Sender<()>,

    /// The hashmap of all actors
    actors: Arc<RwLock<HashMap<String, ActorType<F>>>>,

    /// The notification broadcast
    #[cfg(feature = "notifications")]
    notification: broadcast::Sender<N>,
    #[cfg(not(feature = "notifications"))]
    _notification: PhantomData<N>,
}

/// Implementation of Debug for System
impl<F: Message, N> std::fmt::Debug for System<F, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Do some magic to include certain fields depending on enabled features

        let mut d = f.debug_struct("System");
        
        let d = d.field("id", &self.id);
        
        #[cfg(feature = "foreign")]
        let d = d.field("foreign", &"...");
        
        #[cfg(not(feature = "foreign"))]
        let d = d.field("_phantom", &self._phantom);
        

        let d = d.field("shutdown", &self.shutdown);
        
        let d = d.field("actors",  &"...");
        
        #[cfg(feature = "notifications")]
        let d = d.field("notification", &self.notification);
        
        #[cfg(not(feature = "notifications"))]
        let d = d.field("_notification", &self._notification);
    
        d.finish()
    }
}

#[cfg(feature = "foreign")]
impl<F: Message, N> System<F, N> {
    /// Returns the foreign channel reciever wrapped in an [`Option<T>`].
    /// [`None`] will be returned if the foreign reciever has already been retrieved.
    pub async fn get_foreign(&self) -> Option<mpsc::Receiver<ForeignMessage<F>>> {
        // Lock the foreign reciever
        let mut foreign_reciever = self.foreign.foreign_reciever.lock().await;

        // Return the contents and replace with None
        std::mem::take(std::ops::DerefMut::deref_mut(&mut foreign_reciever))
    }

    /// Returns true if the given [`ActorPath`] is a foreign actor
    pub fn is_foreign(&self, actor: &crate::actor::path::ActorPath) -> bool {
        // If the first system in the actor exists and it it not this system, then it is a foreign system
        actor.first().is_some_and(|v| v != self.id)
    }

    /// Returns true if someone is waiting for a foreign message
    pub async fn can_send_foreign(&self) -> bool {
        self.foreign.foreign_reciever.lock().await.is_none()
    }

    /// Relays a foreign message to this system
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self, foreign)))]
    pub async fn relay_foreign(&self, foreign: ForeignMessage<F>) -> Result<(), ActorError> {
        // Get the target
        let target = foreign.get_target();

        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, system=self.id, "Relaying foreign message to {}", target);
        
        // If it is a foreign actor or the lenth of the systems is larger than 1
        if self.is_foreign(target) || target.systems().len() > 1 {

            #[cfg(feature = "tracing")]
            event!(Level::TRACE, system=self.id, "Determined message is targeted at another system");

            // Pop off the target if the top system is us (ex. "thissystem:foreign:actor")
            let foreign = if self.is_foreign(target) {
                foreign
            } else {
                #[cfg(feature = "tracing")]
                event!(Level::TRACE, system=self.id, "Popped off the current system ID from the path");
                foreign.pop_target()
            };

            // And relay
            self.foreign.foreign_sender
                .send(foreign)
                .await
                .or(Err(ActorError::ForeignSendFail))
        } else {
            #[cfg(feature = "tracing")]
            event!(Level::TRACE, system=self.id, "Determined message is targeted at this system");

            // Send to a local actor
            self.send_foreign_to_local(foreign).await
        }
    }

    /// Sends a foreign message to a local actor
    #[cfg(feature = "federated")]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self, foreign)))]
    async fn send_foreign_to_local(&self, foreign: ForeignMessage<F>) -> Result<(), ActorError> {
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, system=self.id, "Sending foreign message to local actor");

        match foreign {
            ForeignMessage::FederatedMessage(message, responder, target) => {
                #[cfg(feature = "tracing")]
                event!(Level::TRACE, system=self.id, actor=target.actor(), "Foreign message is a federated message");

                // Get actors as read
                let actors = self.actors.read().await;

                // Get the local actor
                let actor = actors.get(&(self.id.clone() + ":" + target.actor()));

                // If it does not exist, then error
                let Some(actor) = actor else {
                    #[cfg(feature = "tracing")]
                    event!(Level::ERROR, system=self.id, actor=target.actor(), "Failed to retrieve actor while processing foreign federated message.");
                    return Err(ActorError::ForeignTargetNotFound);
                };

                // Send the message
                actor
                    .handle_foreign(ForeignMessage::FederatedMessage(message, responder, target))
                    .await
            }
            ForeignMessage::Message(message, responder, target) => {
                #[cfg(feature = "tracing")]
                event!(Level::TRACE, system=self.id, actor=target.actor(), "Foreign message is a regular message");

                // Get actors as read
                let actors = self.actors.read().await;

                // Get the local actor
                let actor = actors.get(&(self.id.clone() + ":" + target.actor()));

                // If it does not exist, then error
                let Some(actor) = actor else {
                    #[cfg(feature = "tracing")]
                    event!(Level::ERROR, system=self.id, actor=target.actor(), "Failed to retrieve actor while processing foreign message.");
                    return Err(ActorError::ForeignTargetNotFound);
                };

                
                // Send the message
                actor
                    .handle_foreign(ForeignMessage::Message(message, responder, target))
                    .await
            }
        }
    }


    #[cfg(not(feature = "federated"))]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self, foreign)))]
    async fn send_foreign_to_local(&self, foreign: ForeignMessage<F>) -> Result<(), ActorError> {
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, system=self.id, "Sending foreign message to local actor. Federated messages are disabled.");

        // Get actors as read
        let actors = self.actors.read().await;

        // Get the local actor
        let actor = actors.get(foreign.get_target().actor());

        // If it does not exist, then error
        let Some(actor) = actor else {
            #[cfg(feature = "tracing")]
            event!(Level::ERROR, system=self.id, actor=foreign.get_target().actor(), "Failed to retrieve actor while processing foreign message.");
            return Err(ActorError::ForeignTargetNotFound);
        };

        
        // Send the message
        actor
            .handle_foreign(foreign)
            .await
    }
}

#[cfg(feature = "foreign")]
#[cfg(feature = "notifications")]
impl<F: Message, N> System<F, N> {
    /// Subscribes to the foreign notification sender
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    pub fn subscribe_foreign_notify(&self) -> broadcast::Receiver<N> {
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, system=self.id, "Party subscribed to the foreign notification sender");

        self.foreign.foreign_notification.subscribe()
    }

    /// Notifies only actors on this sytem
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self, notification)))]
    pub fn notify_local(&self, notification: N) -> usize {
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, system=self.id, "Party sent a notification that is only visible on the local system.");

        self.notification.send(notification).unwrap_or(0)
    }

}


#[cfg(feature = "notifications")]
impl<F: Message, N: Clone> System<F, N> {
    /// Returns a notification reciever associated with the system's notification broadcaster.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    pub(crate) fn subscribe_notify(&self) -> broadcast::Receiver<N> {
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, system=self.id, "Party subscribed to the local notification sender");

        self.notification.subscribe()
    }

    /// Notifies all actors.
    /// Returns the number of actors notified on this sytem
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self, notification)))]
    pub fn notify(&self, notification: N) -> usize {
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, system=self.id, "Sending notification");

        #[cfg(feature = "tracing")]
        event!(Level::TRACE, system=self.id, "Sending notification over foreign channel");

        #[cfg(feature = "foreign")]
        let _ = self.foreign.foreign_notification.send(notification.clone());

        #[cfg(feature = "tracing")]
        event!(Level::TRACE, system=self.id, "Sending notification over local channel");
        self.notification.send(notification).unwrap_or(0)
    }

    /// Yields the current task until all notifications have been recieved
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    pub async fn drain_notify(&self) {
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, system=self.id, "Draining notifications");

        while !self.notification.is_empty() {
            tokio::task::yield_now().await;
        }

        #[cfg(feature = "tracing")]
        event!(Level::TRACE, system=self.id, "Notifications drained");
    }
}



impl<F, N> System<F, N>
where
    F: Message,
    N: Notification,
{
    

    /// Gets the system's id
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Adds an actor to the system
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self, actor, error_policy)))]
    pub async fn add_actor<
        A: Actor + HandleNotification<N> + HandleFederated<F> + HandleMessage<M>,
        M: Message,
    >(
        &self,
        actor: A,
        id: &str,
        error_policy: SupervisorErrorPolicy,
    ) -> Result<LocalHandle<F, M>, SystemError> {
        
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, system=self.id, actor=id, "Adding actor to system");
        
        #[cfg(feature = "foreign")]
        let id = &(self.id.clone() + ":" + id);


        // Convert id to a path
        #[cfg(feature = "foreign")]
        let path = ActorID::new(id).ok_or(SystemError::InvalidPath)?;
        #[cfg(not(feature = "foreign"))]
        let path = id.to_string();

        // Lock write access to the actor map
        let mut actors = self.actors.write().await;

        // If the key is already in actors, return an error
        if actors.contains_key(id) {
            #[cfg(feature = "tracing")]
            event!(Level::ERROR, system=self.id, actor=id, "An actor with the same ID already exists");
            return Err(SystemError::ActorExists);
        }

        // Initialize the supervisor
        let (mut supervisor, handle) = ActorSupervisor::new(actor, path, self, error_policy);

        #[cfg(feature = "tracing")]
        event!(Level::TRACE, system=self.id, actor=id, "Initializing actor supervisor");

        // Clone the system
        let system = self.clone();

        #[cfg(feature = "tracing")]
        event!(Level::TRACE, system=self.id, actor=id, "Spawning supervisor task");

        // Start the supervisor task
        tokio::spawn(async {
            // TODO: Log this.
            let _ = supervisor.run(system).await;
            let _ = supervisor.cleanup().await;

            drop(supervisor);
        });

        #[cfg(feature = "tracing")]
        event!(Level::TRACE, system=self.id, actor=id, "Supervisor task spawned");

        // Insert the handle into the map
        #[cfg(feature = "foreign")]
        let v: Box<dyn ActorEntry<Federated = F> + Send + Sync> = Box::new(handle.clone());
        #[cfg(not(feature = "foreign"))]
        let v: Box<dyn ActorEntry + Send + Sync> = Box::new(handle.clone());
        
        // If foreign messages are disabled, we need to fit some phantom data in here
        #[cfg(not(feature = "foreign"))]
        let v = (v, PhantomData::default());

        actors.insert(id.to_string(), v);

        #[cfg(feature = "tracing")]
        event!(Level::TRACE, system=self.id, actor=id, "Actor handle inserted");

        // Return the handle
        Ok(handle)
    }

    /// Retrieves an actor from the system, returning None if the actor does not exist
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    pub async fn get_actor<M: Message>(&self, id: &str) -> Option<GetActorReturn<F, M>> {

        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, system=self.id, actor=id, "Retrieving actor");

        // Get the actor path
        #[cfg(feature = "foreign")]
        let path = ActorID::new(id)?;
        #[cfg(not(feature = "foreign"))]
        let path = id.to_string();
        
        

        // If the first system exists and it is not this system, then create a foreign actor handle
        #[cfg(feature = "foreign")]
        if path.first().unwrap_or(&self.id) != self.id {
            #[cfg(feature = "tracing")]
            event!(Level::TRACE, system=self.id, actor=id, "Retrieving as a foreign actor handle");

            // Create and return a foreign handle
            return Some(Box::new(ForeignHandle {
                foreign: self.foreign.foreign_sender.clone(),
                system: self.clone(),
                path,
            }));
        }

        #[cfg(feature = "tracing")]
        event!(Level::TRACE, system=self.id, actor=id, "Retrieving as a local actor handle");

        // Lock read access to the actor map
        let actors = self.actors.read().await;


        // Try to get the actor
        #[cfg(feature = "foreign")]
        let actor = actors.get(path.actor())?;

        #[cfg(not(feature = "foreign"))]
        let actor = actors.get(&path)?;

        // If foreign messages are not enabled, get actual actor, not the PhantomData
        #[cfg(not(feature = "foreign"))]
        let actor = &actor.0;

        // Downcast
        let actor = actor.as_any().downcast_ref::<LocalHandle<F, M>>();


        #[cfg(feature = "tracing")]
        if actor.is_some() {
            event!(Level::TRACE, system=self.id, actor=id, "Found local actor");
        } else {
            event!(Level::TRACE, system=self.id, actor=id, "Local actor not found");
        }
        
        // Clone into a box
        match actor {

            Some(v) => {
                let v = v.clone();

                #[cfg(feature = "foreign")]
                let v = Box::new(v);

                Some(v)
            },
            None => None,
        }
    }

    

    
    /// Shutsdown all actors on the system, drains the shutdown channel, and then clears all actors
    ///
    /// # Note
    /// This does not shutdown the system itself, but only actors running on the system. The system is cleaned up at drop.
    /// Even after calling this function, actors can still be added to the system. This returns the number of actors shutdown.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    pub async fn shutdown(&self) -> usize {

        #[cfg(feature = "tracing")]
        event!(Level::INFO, system=self.id, "Shutting down all actors on system");

        // Send the shutdown signal
        let res = self.shutdown.send(()).unwrap_or(0);

        #[cfg(feature = "tracing")]
        event!(Level::TRACE, system=self.id, "Shutdown signal sent");

        // Drain the shutdown list. We do this before removing the actors so that we can ensure actors are properly cleaned up before dropping them.
        // This ensures that there will be no runtime errors due to actors being referenced during a shutdown, but may cause dropped messages.
        // Dropped messages can be fixed by usign request instead of send for any critical messages.
        self.drain_shutdown().await;

        #[cfg(feature = "tracing")]
        event!(Level::TRACE, system=self.id, "Shutdown drained");

        // Borrow the actor map as writable
        let mut actors = self.actors.write().await;

        // Clear the list
        actors.clear();
        actors.shrink_to_fit();

        // Remove the lock
        drop(actors);

        #[cfg(feature = "tracing")]
        event!(Level::TRACE, system=self.id, "All actor handles in system dropped.");

        res
    }

    /// Subscribes to the shutdown reciever
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    pub(crate) fn subscribe_shutdown(&self) -> broadcast::Receiver<()> {
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, system=self.id, "Party subscribed to shutdown notification.");

        self.shutdown.subscribe()
    }

    /// Waits until all actors have shutdown
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    async fn drain_shutdown(&self) {
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, system=self.id, "Draining shutdown");

        while !self.shutdown.is_empty() {
            tokio::task::yield_now().await;
        }

        #[cfg(feature = "tracing")]
        event!(Level::TRACE, system=self.id, "Shutdown drained. All listening actors are offline.");
    }
}

/// Creates a new system with the given id and types for federated messages and notification.
/// Use this function when you are using both federated messages and notifications.
fn new_internal<F: Message, N: Notification>(id: &str) -> System<F, N> {
    

    // Create the notification sender
    #[cfg(feature = "notifications")]
    let (notification, _) = broadcast::channel(64);

    

    // Create the shutdown sender
    let (shutdown, _) = broadcast::channel(8);

    // If the foreign feature is enabled, create the struct containing the foreign internals
    #[cfg(feature = "foreign")]
    let foreign = {
        // Create the foreign channel
        let (foreign_sender, foreign_reciever) = mpsc::channel(64);

        // Create the foreign notification sender
        let (foreign_notification, _) = broadcast::channel(64);

        foreign::ForeignComponents {
            foreign_notification,
            foreign_reciever: Arc::new(Mutex::new(Some(foreign_reciever))),
            foreign_sender
        }
    };

    System {
        id: id.to_string(),
        #[cfg(feature = "foreign")]
        foreign,
        #[cfg(not(feature="foreign"))]
        _phantom: PhantomData::default(),
        shutdown,
        actors: Default::default(),
        #[cfg(feature = "notifications")]
        notification,
        #[cfg(not(feature = "notifications"))]
        _notification: PhantomData::default(),
    }
}

#[cfg(feature = "notifications")]
#[cfg(feature = "federated")]
pub fn new<F: Message, N: Notification>(id: &str) -> System<F, N> {
    new_internal(id)
}

#[cfg(not(feature = "notifications"))]
#[cfg(not(feature = "federated"))]
/// Creates a new system that does not use federated messages or notifications
pub fn new(id: &str) -> System<crate::message::DefaultFederated, crate::message::DefaultNotification> {
    new_internal(id)
}

#[cfg(not(feature = "notifications"))]
#[cfg(feature = "federated")]
/// Creates a new system that uses federated messages but not notifications.
pub fn new<F: Message>(id: &str) -> System<F, crate::message::DefaultNotification> {
    new_internal(id)
}

#[cfg(feature = "notifications")]
#[cfg(not(feature = "federated"))]
/// Creates a new system that uses notifications but not federated messages
pub fn new<N: Notification>(id: &str) -> System<crate::message::DefaultFederated, N> {
    new_internal(id)
}