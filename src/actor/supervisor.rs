use tokio::sync::broadcast;

use crate::{system::{System, SystemNotification}, actor::NotifyHandler};

use super::{ActorMetadata, Actor, ActorID, handle::ActorHandle, context::ActorContext};



/// # ActorSupervisor
/// This struct is responsible for handling a specific actor's entire lifecycle, including initialization, message processing, an deinitialization.
pub struct ActorSupervisor<A: Actor, N: SystemNotification> {
    /// Contains the metadata of the running actor.
    metadata: ActorMetadata,
    /// Stores the actual actor
    actor: A,
    /// Stores the notification reciever
    notify_reciever: broadcast::Receiver<N>
}

impl<A: Actor + NotifyHandler<N>, N: SystemNotification> ActorSupervisor<A, N> {

    /// Creates a new supervisor from an actor, id, and system
    pub fn new(actor: A, id: ActorID, system: System<N>) -> (ActorSupervisor<A, N>, ActorHandle) {

        // Subscribe to the notification reciever
        let notify_reciever = system.subscribe_notify();

        // Create the actor metadata
        let metadata = ActorMetadata {
            id
        };

        // Create the supervisor
        let supervisor = ActorSupervisor {
            metadata: metadata.clone(),
            actor, 
            notify_reciever
        };

        // Create the handle
        let handle = ActorHandle::new(metadata);


        (supervisor, handle)
    }

    /// Returns a reference to the actor's metadata
    pub fn get_metadata(&self) -> &ActorMetadata {
        &self.metadata
    }

    /// This function handles the lifecycle of the actor. 
    /// This should be run in a separate task.
    /// 
    /// # Panics
    /// This function panics any error is returned from the actor. This will be replaced with better error handling later.
    pub async fn run(&mut self, system: System<N>) {

        // Create an actor context
        let mut context = ActorContext {

        };

        // Initialize the actor
        self.actor.initialize(&mut context).await.unwrap();

        // Continuously recieve messages
        loop {
            tokio::select! {
                notification = self.notify_reciever.recv() => {

                    // When we recieve a notification, pass it along to the actor handler
                    self.actor.notified(&mut context, notification.unwrap()).await.unwrap();
                }
            }
        }

        // Deinitialize the actor
        self.actor.deinitialize(&mut context).await.unwrap();
    }
}