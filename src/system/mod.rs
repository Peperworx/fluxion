//! The implementation of systems and surrounding types

use std::{sync::Arc, mem, collections::HashMap};

use tokio::sync::{mpsc, Mutex, RwLock, broadcast};

use crate::{message::{foreign::{ForeignMessage}, Notification, Message}, error::ActorError, actor::{path::ActorPath, ActorEntry}};


/// # System
/// The core part of Fluxion, the [`System`] runs actors and handles communications between other systems.
/// 
/// ## Inter-System Communication
/// Fluxion systems enable communication by having what is called a foreign channel.
/// The foreign channel is an mpsc channel, the Reciever for which can be retrieved once by a single outside source using [`System::get_foreign`].
/// When a Message or Foreign Message is sent to an external actor, or a Notification is sent at all, the foreign
/// channel will be notified.
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
    /// 
    /// ## Notification Propagation
    /// Notifications should be propagated in a special way. Each notification does have a target actor associated
    /// with it, but the actor ID is blank. The notification message will be sent along the entire chain of systems contained in the
    /// [`ActorPath`]. When the final system is reached, it will send the Notification message to every foreign system that is has a direct
    /// connection to, with an empty system path. This will cause all of these systems to trigger the Notification.
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
            todo!()
        }
    }

    

}
