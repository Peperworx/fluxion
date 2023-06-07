use std::{any::Any, collections::HashMap, sync::Arc};

use tokio::sync::{RwLock, broadcast};

use crate::{actor::{ActorID, handle::ActorHandle, Actor, supervisor::ActorSupervisor, NotifyHandler}, error::{SystemError, ErrorPolicyCollection}};

/// # SystemNotification
/// This marker trait should be implemented for any type which will be used as a system notification.
/// This is by default implemented for all types that are `Clone + Send + Sync + 'static`
pub trait SystemNotification: Clone + Send + Sync + 'static {}

impl<T> SystemNotification for T where T: Clone + Send + Sync + 'static {}

/// # ActorType
/// This type is shorthand for dyn Any + Send + Sync + 'static, which is the type that actors are stored in the system as.
type ActorType = dyn Any + Send + Sync + 'static;

/// # SystemID
/// This type (currently set to `String`) is used to identify a system. This may come in handy while running applications
/// with multiple systems or communication over the network
pub type SystemID = String;

/// # System
/// The System manages all actors, messages, notifications, and events.
#[derive(Clone)]
pub struct System<N: SystemNotification> {
    id: SystemID,
    /// This is a Arc to a RwLock of a HashMap containing the actors. This allows the system to be repeatedly cloned while
    /// still being able to access the same map of actors.
    actors: Arc<RwLock<HashMap<ActorID, Box<ActorType>>>>,
    /// This broadcast Sender is used to send a notification to all actors.
    notify_sender: broadcast::Sender<N>
}

impl<N: SystemNotification> System<N> {

    /// Creates a new system with SystemID id
    pub fn new(id: SystemID) -> Self {
        // Create a new notification sender
        let (notify_sender, _) = broadcast::channel(1024);

        Self {
            id,
            actors: Default::default(),
            notify_sender
        }
    }

    /// Retrieves the system's id
    pub fn get_id(&self) -> SystemID {
        self.id.clone()
    }

    /// Adds an actor to the system
    pub async fn add_actor<A: Actor + NotifyHandler<N>>(&self, actor: A, id: ActorID, error_policy: ErrorPolicyCollection) -> Result<ActorHandle, SystemError> {

        // Lock write access to the actor map
        let mut actors = self.actors.write().await;

        // If the key is already in actors, return an error
        if actors.contains_key(&id) {
            return Err(SystemError::ActorAlreadyExists);
        }

        // Clone the system (self)
        let system = self.clone();

        // Initialize the supervisor
        let (mut supervisor, handle) = ActorSupervisor::new(actor, id.clone(), system.clone(), error_policy);

        // Start the supervisor task
        tokio::spawn(async move {
            supervisor.run(system).await;
        });

        // Insert the handle into the map
        actors.insert(id, Box::new(handle.clone()));

        // Return the handle
        Ok(handle)
    }

    /// Retrieves an actor from the system, returning None if the actor does not exist
    pub async fn get_actor(&self, id: &ActorID) -> Option<ActorHandle> {

        // Lock read access to the actor map
        let actors = self.actors.read().await;

        // Try to get the actor
        let actor = actors.get(id);

        // If the actor exists, downcast it to ActorHandle<M>. If the actor does not exist, return None
        actor.and_then(|boxed| {
            boxed.downcast_ref().cloned()
        })
    }

    /// Broadcasts a notification to all actors, and returns the number of actors that recieved the notification.
    /// 
    /// # Note
    /// Beneath this function, Fluxion uses a tokio broadcast channel. When Recievers exist, the broadcast channel's send function returns an error.
    /// This function abstracts away the error, and returns 0 if there are no listeners. This is because there may be some points in time when no actors exist,
    /// but there will be actors in the future.
    pub fn notify(&self, notification: N) -> usize {
        if let Ok(v) = self.notify_sender.send(notification) {
            v
        } else {
            0
        }
    }

    /// Returns a notification Reciever for recieving notifications
    pub fn subscribe_notify(&self) -> broadcast::Receiver<N> {
        self.notify_sender.subscribe()
    }

    /// Yields the current task until all notifications have been recieved
    pub async fn drain_notify(&self) {
        while self.notify_sender.len() > 0 {
            tokio::task::yield_now().await;
        }
    }
}