//! The implementation of systems and surrounding types

use std::{sync::Arc, mem, collections::HashMap};

use tokio::sync::{mpsc, Mutex, RwLock, broadcast};

use crate::{message::{foreign::{ForeignMessage}, Notification, Message, handler::HandleNotification}, error::{ActorError, SystemError}, actor::{path::ActorPath, ActorEntry, Actor, supervisor::{ActorSupervisor, SupervisorErrorPolicy}, handle::ActorHandle}};


/// # System
/// The core part of Fluxion, the [`System`] runs actors and handles communications between other systems.
/// 
/// ## Inter-System Communication
/// Fluxion systems enable communication by having what is called a foreign channel.
/// The foreign channel is an mpsc channel, the Reciever for which can be retrieved once by a single outside source using [`System::get_foreign`].
/// When a Message or Foreign Message is sent to an external actor, or a Notification is sent at all, the foreign
/// channel will be notified.
#[derive(Clone)]
pub struct System<F, N>
where
    F: Message,
    N: Notification, {

    /// The id of the system
    id: String,

    /// The reciever for foreign messages
    /// This uses a Mutex to provide interior mutability.
    foreign_reciever: Arc<Mutex<Option<mpsc::Receiver<ForeignMessage<F, N>>>>>,
    
    /// The sender for foreign messages
    foreign_sender: mpsc::Sender<ForeignMessage<F, N>>,

    /// The hashmap of all actors
    actors: Arc<RwLock<HashMap<String, Box<dyn ActorEntry<Federated = F, Notification = N>>>>>,

    /// The notification broadcast
    notification: broadcast::Sender<N>,
}

impl<F, N> System<F, N>
where
    F: Message,
    N: Notification {

    /// Creates a new system with the given id.
    pub fn new(id: &str) -> Self {
        // Create the foreign channel
        let (foreign_sender, foreign_reciever) = mpsc::channel(16);

        // Create the notification sender
        let (notification, _) = broadcast::channel(16);

        Self {
            id: id.to_string(),
            foreign_reciever: Arc::new(Mutex::new(Some(foreign_reciever))),
            foreign_sender,
            actors: Default::default(),
            notification
        }
    }
    
    /// Returns the foreign channel reciever wrapped in an [`Option<T>`].
    /// [`None`] will be returned if the foreign reciever has already been retrieved.
    pub async fn get_foreign(&self) -> Option<mpsc::Receiver<ForeignMessage<F, N>>> {
        
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

    /// Relays a foreign message to this system
    pub async fn relay_foreign(&self, foreign: ForeignMessage<F, N>) -> Result<(), ActorError> {
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
            self.foreign_sender.send(foreign).await.or(Err(ActorError::ForeignSendFail))
        } else {
            // Send to a local actor
            self.send_foreign_to_local(foreign).await
        }
    }

    /// Sends a foreign message to a local actor
    async fn send_foreign_to_local(&self, foreign: ForeignMessage<F, N>) -> Result<(), ActorError> {
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
                actor.handle_foreign(ForeignMessage::FederatedMessage(message, responder, target)).await
            },
            ForeignMessage::Notification(notification, _) => {
                // Send the notification on this system
                self.notify(notification).await;

                Ok(())
            },
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
                actor.handle_foreign(ForeignMessage::Message(message, responder, target)).await
            },
        }
    }

    /// Adds an actor to the system
    pub async fn add_actor<A: Actor + HandleNotification<N>, M: Message>(&self, actor: A, id: &str, error_policy: SupervisorErrorPolicy) -> Result<ActorHandle<F, N, M>, SystemError> {

        // Lock write access to the actor map
        let mut actors = self.actors.write().await;

        // If the key is already in actors, return an error
        if actors.contains_key(id) {
            return Err(SystemError::ActorExists);
        }

        // Initialize the supervisor
        let (mut supervisor, handle) = ActorSupervisor::new(actor, id, self.clone(), error_policy);
        
        // Start the supervisor task
        tokio::spawn(async move {
            // TODO: Log this.
            let _ = supervisor.run().await;
            let _ = supervisor.cleanup().await;
            
            drop(supervisor);
        });

        // Insert the handle into the map
        actors.insert(id.to_string(), Box::new(handle.clone()));

        // Return the handle
        Ok(handle)
    }

    
    /// Returns a notification reciever associated with the system's notification broadcaster.
    pub(crate) fn subscribe_notify(&self) -> broadcast::Receiver<N> {
        self.notification.subscribe()
    }

    /// Notifies all actors on this system.
    /// Returns the number of actors notified
    pub async fn notify(&self, notification: N) -> usize {
        self.notification.send(notification).unwrap_or(0)
    }

    /// Yields the current task until all notifications have been recieved
    pub async fn drain_notify(&self) {
        while !self.notification.is_empty() {
            tokio::task::yield_now().await;
        }
    }
}
