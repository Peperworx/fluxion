use tokio::sync::{broadcast, mpsc, oneshot};

use crate::{system::{System, SystemNotification}, actor::NotifyHandler, error::{ErrorPolicyCollection, ActorError}, handle_policy};

use super::{ActorMetadata, Actor, ActorID, handle::ActorHandle, context::ActorContext, ActorMessage, FederatedHandler};



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
    /// Stores the message reciever
    federated_reciever: mpsc::Receiver<(F, Option<oneshot::Sender<F::Response>>)>,
}

impl<N: SystemNotification, A: Actor + NotifyHandler<N> + FederatedHandler<F>, F: ActorMessage> ActorSupervisor<A, N, F> {

    /// Creates a new supervisor from an actor, id, and system
    pub fn new(actor: A, id: ActorID, system: System<N, F>, error_policy: ErrorPolicyCollection) -> (ActorSupervisor<A, N, F>, ActorHandle<F>) {


        // Subscribe to the notification reciever
        let notify_reciever = system.subscribe_notify();

        // Subscribe to the shutdown reciever
        let shutdown_reciever = system.subscribe_shutdown();

        // Create the actor metadata
        let metadata = ActorMetadata {
            id,
            error_policy,
        };

        // Create a channel for federated messages
        let (federated_sender, federated_reciever) = mpsc::channel(16);


        // Create the supervisor
        let supervisor = ActorSupervisor {
            metadata: metadata.clone(),
            actor,
            system,
            notify_reciever,
            shutdown_reciever,
            federated_reciever
        };

        // Create the handle
        let handle = ActorHandle::new(
            metadata,
            federated_sender
        );


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
        'event: loop {

            tokio::select! {
                _ = self.shutdown_reciever.recv() => {
                    // Just shutdown no matter what happened
                    break 'event;
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
                        if let Ok(n) = n {
                            // Handle policy for the notification handler
                            let handled = handle_policy!(
                                self.actor.notified(&mut context, n.clone()).await,
                                |_| self.metadata.error_policy.notify_handler,
                                (), ActorError
                            ).await;

                            // If error, exit
                            if handled.is_err() {
                                break 'event;
                            }
                        }
                    } else {
                        // Stop the actor if not ok
                        break 'event;
                    }
                },
                federated_message = handle_policy!(
                    self.federated_reciever.recv().await.ok_or(ActorError::OutOfMessages),
                    |_| self.metadata.error_policy.federated_closed,
                    (F, Option<oneshot::Sender<F::Response>>), ActorError
                ) => {

                    // If it is an error, stop the actor
                    let Ok(reciever_result) = federated_message else {
                        break 'event;
                    };

                    // If the result from the reciever is an error, continue because the policy succeeded.
                    let Ok(message) = reciever_result else {
                        continue;
                    };

                    // Call the handler
                    let handler_policy_result = handle_policy!(
                        self.actor.federated_message(&mut context, message.0.clone()).await,
                        |_| self.metadata.error_policy.federated_handler,
                        F::Response, ActorError
                    ).await;
                    
                    // If it is an error, then break
                    let Ok(handler_result) = handler_policy_result else {
                        break 'event;
                    };

                    // If the handler's result is an error, continue because the policy succeeded.
                    let Ok(response) = handler_result else {
                        continue;
                    };

                    
                    // If we shouldn't respond, continue to the next loop iteration
                    let Some(sender) = message.1 else {
                        continue;
                    };
                    
                    // Otherwise,
                    // Send the oneshot
                    let e = sender.send(response);

                    // Special error policy handling. Because of the one shot, Retry will be treated the same as Shutdown
                    if e.is_err() {
                        match self.metadata.error_policy.federated_respond {
                            crate::error::ErrorPolicy::Ignore => {
                                continue;
                            },
                            crate::error::ErrorPolicy::Retry(_) => {
                                break 'event;
                            },
                            crate::error::ErrorPolicy::Shutdown => {
                                break 'event;
                            },
                        }   
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

    pub async fn cleanup(&mut self) {
        // Close all channels
        self.federated_reciever.close();
        
    }
}
