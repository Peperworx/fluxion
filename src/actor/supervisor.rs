use tokio::sync::{broadcast, mpsc};

use crate::{system::{System, SystemNotification}, actor::NotifyHandler, error::{ErrorPolicyCollection, ErrorPolicy, ActorError}, handle_policy};

use super::{ActorMetadata, Actor, ActorID, handle::ActorHandle, context::ActorContext, ActorMessage, MessageType, FederatedHandler};



/// # ActorSupervisor
/// This struct is responsible for handling a specific actor's entire lifecycle, including initialization, message processing, an deinitialization.
pub struct ActorSupervisor<A: Actor + NotifyHandler<N>, N: SystemNotification> {
    /// Contains the metadata of the running actor.
    metadata: ActorMetadata,
    /// Stores the actual actor
    actor: A,
    /// Stores the notification reciever
    notify_reciever: broadcast::Receiver<N>,
}

impl<N: SystemNotification, A: Actor + NotifyHandler<N>> ActorSupervisor<A, N> {

    /// Creates a new supervisor from an actor, id, and system
    pub fn new<F: ActorMessage>(actor: A, id: ActorID, system: System<N, F>, error_policy: ErrorPolicyCollection) -> (ActorSupervisor<A, N>, ActorHandle) {


        // Subscribe to the notification reciever
        let notify_reciever = system.subscribe_notify();

        // Create the actor metadata
        let metadata = ActorMetadata {
            id,
            error_policy,
        };

        // Create the supervisor
        let supervisor = ActorSupervisor {
            metadata: metadata.clone(),
            actor, 
            notify_reciever,
        };

        // Create the handle
        let handle = ActorHandle {
            metadata,
        };


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
    pub async fn run<F: ActorMessage>(&mut self, _system: System<N, F>) {

        // Create an actor context
        let mut context = ActorContext {
            metadata: self.metadata.clone()
        };

        // Initialize the actor, following initialization retry error policy
        let a = handle_policy!(
            self.actor.initialize(&mut context).await,
            self.metadata.error_policy.initialize,
            (), ActorError).await;
        
        // If error, exit
        // TODO: Log
        if a.is_err() {
            return;
        }

        // If ok, just ignore.


        // Continuously recieve messages
        'event: loop {
           break;
        }

        // Deinitialize the actor, following deinitialization retry error policy
        let _ = handle_policy!(
            self.actor.initialize(&mut context).await,
            self.metadata.error_policy.initialize,
            (), ActorError).await;
        
        // We dont even care if it was ok or error until logging is implemented.
    }
}