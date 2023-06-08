use tokio::sync::broadcast;

use crate::{system::{System, SystemNotification}, actor::NotifyHandler, error::{ErrorPolicyCollection, ActorError}, handle_policy};

use super::{ActorMetadata, Actor, ActorID, handle::ActorHandle, context::ActorContext, ActorMessage};



/// # ActorSupervisor
/// This struct is responsible for handling a specific actor's entire lifecycle, including initialization, message processing, an deinitialization.
pub struct ActorSupervisor<A: Actor + NotifyHandler<N>, N: SystemNotification, F: ActorMessage> {
    /// Contains the metadata of the running actor.
    metadata: ActorMetadata,
    /// Stores the actual actor
    actor: A,
    /// The system
    system: System<N, F>,
    /// Stores the notification reciever
    notify_reciever: broadcast::Receiver<N>,
    /// Stores the shutdown reciever
    shutdown_reciever: broadcast::Receiver<()>,
}

impl<N: SystemNotification, A: Actor + NotifyHandler<N>, F: ActorMessage> ActorSupervisor<A, N, F> {

    /// Creates a new supervisor from an actor, id, and system
    pub fn new(actor: A, id: ActorID, system: System<N, F>, error_policy: ErrorPolicyCollection) -> (ActorSupervisor<A, N, F>, ActorHandle) {


        // Subscribe to the notification reciever
        let notify_reciever = system.subscribe_notify();

        // Subscribe to the shutdown reciever
        let shutdown_reciever = system.subscribe_shutdown();

        // Create the actor metadata
        let metadata = ActorMetadata {
            id,
            error_policy,
        };

        // Create the supervisor
        let supervisor = ActorSupervisor {
            metadata: metadata.clone(),
            actor,
            system,
            notify_reciever,
            shutdown_reciever
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
    pub async fn run(&mut self) {

        // Create an actor context
        let mut context = ActorContext {
            metadata: self.metadata.clone()
        };

        // Initialize the actor, following initialization retry error policy
        let a = handle_policy!(
            self.actor.initialize(&mut context).await,
            |_| self.metadata.error_policy.initialize,
            (), ActorError).await;
        
        // If error, exit
        // TODO: Log
        if a.is_err() {
            return;
        }

        // If ok, just ignore.


        // Continuously recieve messages
        loop {

            tokio::select! {
                _ = self.shutdown_reciever.recv() => {
                    // Just shutdown no matter what happened
                    break;
                },
                notification = handle_policy!(
                    self.notify_reciever.recv().await,
                    |e| match e {
                        broadcast::error::RecvError::Closed => self.metadata.error_policy.notify_closed,
                        broadcast::error::RecvError::Lagged(_) => self.metadata.error_policy.notify_lag,
                    },
                    N, broadcast::error::RecvError
                ) => {
                    // If Ok, then continue on with operation
                    if let Ok(n) = notification {
                        // If n is Ok, handle. If not, just continue
                        match n {
                            Ok(n) => {
                                
                                // Handle policy for the notification handler
                                let handled = handle_policy!(
                                    self.actor.notified(&mut context, n).await,
                                    |_| self.metadata.error_policy.notify_handler,
                                    (), ActorError
                                ).await;

                                // If error, exit
                                if handled.is_err() {
                                    break;
                                }
                            },
                            Err(e) => {
                                println!("{:?}", e);
                            }
                        }
                    } else {
                        // Stop the actor if not ok
                        break;
                    }
                }
            }
        }

        // Deinitialize the actor, following deinitialization retry error policy
        let _ = handle_policy!(
            self.actor.deinitialize(&mut context).await,
            |_| self.metadata.error_policy.deinitialize,
            (), ActorError).await;
        
        // We dont even care if it was ok or error until logging is implemented.
    }
}