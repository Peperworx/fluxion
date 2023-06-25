//! The implementation of systems and surrounding types

use std::{collections::HashMap, mem, sync::Arc};

use tokio::sync::{broadcast, mpsc, Mutex, RwLock};

use crate::{
    actor::{
        handle::{ActorHandle, foreign::ForeignHandle, local::LocalHandle},
        path::ActorPath,
        supervisor::{ActorSupervisor, SupervisorErrorPolicy},
        Actor, ActorEntry,
    },
    error::{ActorError, SystemError},
    message::{
        foreign::ForeignMessage,
        handler::{HandleFederated, HandleMessage, HandleNotification},
        Message, Notification, DefaultNotification, DefaultFederated,
    },
};

/// # ActorType
/// The type of an actor in the hashmap
type ActorType<F> = Box<dyn ActorEntry<Federated = F> + Send + Sync + 'static>;

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
    N: Notification,
{
    /// The id of the system
    id: String,

    /// The reciever for foreign messages
    /// This uses a Mutex to provide interior mutability.
    foreign_reciever: Arc<Mutex<Option<mpsc::Receiver<ForeignMessage<F>>>>>,

    /// The sender for foreign messages
    foreign_sender: mpsc::Sender<ForeignMessage<F>>,

    /// A shutdown sender which tells all actors to stop
    shutdown: broadcast::Sender<()>,

    /// The hashmap of all actors
    actors: Arc<RwLock<HashMap<String, ActorType<F>>>>,

    /// The notification broadcast
    notification: broadcast::Sender<N>,

    /// The foreign notification sender
    foreign_notification: broadcast::Sender<N>,
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

    /// Returns the foreign channel reciever wrapped in an [`Option<T>`].
    /// [`None`] will be returned if the foreign reciever has already been retrieved.
    pub async fn get_foreign(&self) -> Option<mpsc::Receiver<ForeignMessage<F>>> {
        // Lock the foreign reciever
        let mut foreign_reciever = self.foreign_reciever.lock().await;

        // Return the contents and replace with None
        mem::take(std::ops::DerefMut::deref_mut(&mut foreign_reciever))
    }

    /// Returns true if the given [`ActorPath`] is a foreign actor
    pub fn is_foreign(&self, actor: &ActorPath) -> bool {
        // If the first system in the actor exists and it it not this system, then it is a foreign system
        actor.first().is_some_and(|v| v != self.id)
    }

    /// Returns true if someone is waiting for a foreign message
    pub async fn can_send_foreign(&self) -> bool {
        self.foreign_reciever.lock().await.is_none()
    }

    /// Relays a foreign message to this system
    pub async fn relay_foreign(&self, foreign: ForeignMessage<F>) -> Result<(), ActorError> {
        // Get the target
        let target = foreign.get_target();

        // If it is a foreign actor or the lenth of the systems is larger than 1
        if self.is_foreign(target) || target.systems().len() > 1 {
            // Pop off the target if the top system is us (ex. "thissystem:foreign:actor")
            let foreign = if self.is_foreign(target) {
                foreign
            } else {
                foreign.pop_target()
            };

            // And relay
            self.foreign_sender
                .send(foreign)
                .await
                .or(Err(ActorError::ForeignSendFail))
        } else {
            // Send to a local actor
            self.send_foreign_to_local(foreign).await
        }
    }

    /// Sends a foreign message to a local actor
    async fn send_foreign_to_local(&self, foreign: ForeignMessage<F>) -> Result<(), ActorError> {
        match foreign {
            ForeignMessage::FederatedMessage(message, responder, target) => {
                // Get actors as read
                let actors = self.actors.read().await;

                // Get the local actor
                let actor = actors.get(target.actor());

                // If it does not exist, then error
                let Some(actor) = actor else {
                    return Err(ActorError::ForeignTargetNotFound);
                };

                // Send the message
                actor
                    .handle_foreign(ForeignMessage::FederatedMessage(message, responder, target))
                    .await
            }
            ForeignMessage::Message(message, responder, target) => {
                // Get actors as read
                let actors = self.actors.read().await;

                // Get the local actor
                let actor = actors.get(target.actor());

                // If it does not exist, then error
                let Some(actor) = actor else {
                    return Err(ActorError::ForeignTargetNotFound);
                };

                // Send the message
                actor
                    .handle_foreign(ForeignMessage::Message(message, responder, target))
                    .await
            }
        }
    }

    /// Adds an actor to the system
    pub async fn add_actor<
        A: Actor + HandleNotification<N> + HandleFederated<F> + HandleMessage<M>,
        M: Message,
    >(
        &self,
        actor: A,
        id: &str,
        error_policy: SupervisorErrorPolicy,
    ) -> Result<LocalHandle<F, M>, SystemError> {
        // Convert id to a path
        let path = ActorPath::new(&(self.id.clone() + ":" + id)).ok_or(SystemError::InvalidPath)?;

        // Lock write access to the actor map
        let mut actors = self.actors.write().await;

        // If the key is already in actors, return an error
        if actors.contains_key(id) {
            return Err(SystemError::ActorExists);
        }

        // Initialize the supervisor
        let (mut supervisor, handle) = ActorSupervisor::new(actor, path, self, error_policy);

        // Clone the system
        let system = self.clone();

        // Start the supervisor task
        tokio::spawn(async {
            // TODO: Log this.
            let _ = supervisor.run(system).await;
            let _ = supervisor.cleanup().await;

            drop(supervisor);
        });

        // Insert the handle into the map
        actors.insert(id.to_string(), Box::new(handle.clone()));

        // Return the handle
        Ok(handle)
    }

    /// Retrieves an actor from the system, returning None if the actor does not exist
    pub async fn get_actor<M: Message>(&self, id: &str) -> Option<Box<dyn ActorHandle<F, M>>> {
        // Get the actor path
        let path = ActorPath::new(id)?;

        // If the first system exists and it is not this system, then create a foreign actor handle
        if path.first().unwrap_or(&self.id) != self.id {
            // Create and return a foreign handle
            return Some(Box::new(ForeignHandle {
                foreign: self.foreign_sender.clone(),
                system: self.clone(),
                path,
            }));
        }

        // Lock read access to the actor map
        let actors = self.actors.read().await;

        // Try to get the actor
        let actor = actors.get(path.actor())?;

        // Downcast and clone into a box.
        match actor.as_any().downcast_ref::<LocalHandle<F, M>>() {
            Some(v) => Some(Box::new(v.clone())),
            None => None,
        }
    }

    /// Returns a notification reciever associated with the system's notification broadcaster.
    pub(crate) fn subscribe_notify(&self) -> broadcast::Receiver<N> {
        self.notification.subscribe()
    }

    /// Notifies all actors.
    /// Returns the number of actors notified on this sytem
    pub fn notify(&self, notification: N) -> usize {
        let _ = self.foreign_notification.send(notification.clone());
        self.notification.send(notification).unwrap_or(0)
    }

    /// Yields the current task until all notifications have been recieved
    pub async fn drain_notify(&self) {
        while !self.notification.is_empty() {
            tokio::task::yield_now().await;
        }
    }

    /// Subscribes to the foreign notification sender
    pub fn subscribe_foreign_notify(&self) -> broadcast::Receiver<N> {
        self.foreign_notification.subscribe()
    }

    /// Notifies only actors on this sytem
    pub fn notify_local(&self, notification: N) -> usize {
        self.notification.send(notification).unwrap_or(0)
    }

    /// Shutsdown all actors on the system, drains the shutdown channel, and then clears all actors
    ///
    /// # Note
    /// This does not shutdown the system itself, but only actors running on the system. The system is cleaned up at drop.
    /// Even after calling this function, actors can still be added to the system. This returns the number of actors shutdown.
    pub async fn shutdown(&self) -> usize {
        // Send the shutdown signal
        let res = self.shutdown.send(()).unwrap_or(0);

        // Borrow the actor map as writable
        let mut actors = self.actors.write().await;

        // Clear the list
        actors.clear();
        actors.shrink_to_fit();

        // Remove the lock
        drop(actors);

        // Drain the shutdown list
        self.drain_shutdown().await;

        res
    }

    /// Subscribes to the shutdown reciever
    pub(crate) fn subscribe_shutdown(&self) -> broadcast::Receiver<()> {
        self.shutdown.subscribe()
    }

    /// Waits until all actors have shutdown
    async fn drain_shutdown(&self) {
        while !self.shutdown.is_empty() {
            tokio::task::yield_now().await;
        }
    }
}

/// Creates a new system with the given id and types for federated messages and notification.
/// Use this function when you are using both federated messages and notifications.
pub fn new<F: Message, N: Notification>(id: &str) -> System<F, N> {
    // Create the foreign channel
    let (foreign_sender, foreign_reciever) = mpsc::channel(64);

    // Create the notification sender
    let (notification, _) = broadcast::channel(64);

    // Create the foreign notification sender
    let (foreign_notification, _) = broadcast::channel(64);

    // Create the shutdown sender
    let (shutdown, _) = broadcast::channel(8);

    System {
        id: id.to_string(),
        foreign_reciever: Arc::new(Mutex::new(Some(foreign_reciever))),
        foreign_sender,
        shutdown,
        actors: Default::default(),
        notification,
        foreign_notification
    }
}

/// Creates a new system that does not use federated messages or notifications
pub fn new_none(id: &str) -> System<DefaultFederated, DefaultNotification> {
    new(id)
}


/// Creates a new system that uses federated messages but not notifications.
pub fn new_federated<F: Message>(id: &str) -> System<F, DefaultNotification> {
    new(id)
}

/// Creates a new system that uses notifications but not federated messages
pub fn new_notifications<N: Notification>(id: &str) -> System<DefaultFederated, N> {
    new(id)
}