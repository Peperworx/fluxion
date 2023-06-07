use tokio::sync::broadcast;

use crate::{system::{System, SystemNotification}, actor::NotifyHandler, error::{ErrorPolicyCollection, ErrorPolicy}};

use super::{ActorMetadata, Actor, ActorID, handle::ActorHandle, context::ActorContext};



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
    pub fn new(actor: A, id: ActorID, system: System<N>, error_policy: ErrorPolicyCollection) -> (ActorSupervisor<A, N>, ActorHandle) {

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
    pub async fn run(&mut self, _system: System<N>) {

        // Create an actor context
        let mut context = ActorContext {
            metadata: self.metadata.clone()
        };

        // Initialize the actor, following initialization retry error policy
        {
            let mut retry_count = 0;

            while self.actor.initialize(&mut context).await.is_err() {
                // Match on the error policy
                match self.metadata.error_policy.initialize {
                    ErrorPolicy::Ignore => {
                        // Ignore the error
                        break;
                    },
                    ErrorPolicy::Retry(ct) => {
                        // If retry count has been reached, then fail, shutting down the actor
                        if retry_count >= ct {
                            // Todo: logging here.
                            return;
                        }

                        // If not, increment and try again
                        retry_count += 1;
                        continue;
                    },
                    // Treat any other as ignore
                    _ => break
                }
            }
        }

        // Continuously recieve messages
        'event: loop {
            // This returns an (Option<N>, bool). If bool is true and option is Some, then the operation succeeded. If bool is true and option is None, the operation was ignored.
            // If both are false and None, the operation failed.
            let notify_recieve = async {
                let mut retry_count = 0;
                
                loop {
                    // Get the result
                    let r = self.notify_reciever.recv().await;

                    // If the result is ok, return
                    if let Ok(v) = r {
                        return (Some(v), true);
                    }

                    if let Err(e) = r {
                        // Select the policy based on error type
                        let policy = match e {
                            broadcast::error::RecvError::Closed => self.metadata.error_policy.notify_closed,
                            broadcast::error::RecvError::Lagged(_) => self.metadata.error_policy.notify_lag,
                        };

                        // Match on the error policy
                        match policy {
                            ErrorPolicy::Ignore => {
                                // Ignore the error
                                return (None, true);
                            },
                            ErrorPolicy::Retry(ct) => {
                                // If retry count has been reached, then fail, shutting down the actor
                                if retry_count >= ct {
                                    return (None, false);
                                }

                                // If not, increment and try again
                                retry_count += 1;
                                continue;
                            },
                            ErrorPolicy::Shutdown => {
                                // Return none and false to indicate shutdown
                                return (None, false);
                            }
                        }
                    }
                }
            };

            tokio::select! {
                notification = notify_recieve => {

                    // If failed, exit
                    if !notification.1 {
                        break;
                    }

                    // If some, nofity
                    if let Some(n) = notification.0 {
                        // When we recieve a notification, pass it along to the actor handler.
                        // Handle the error policy here

                        // Create a retry count
                        let mut retry_count = 0;

                        // Loop until either success or failure
                        loop {
                            
                            // Get the result of the notified actor
                            // Todo: find potential restructure to get rid of this clone.
                            let r = self.actor.notified(&mut context, n.clone()).await;

                            // If ok, break
                            if r.is_ok() {
                                break;
                            }

                            // If error, resolve policy
                            if r.is_err() {
                                // Match on the error policy
                                match self.metadata.error_policy.notify_handler {
                                    ErrorPolicy::Ignore => {
                                        // Ignore the error
                                        break;
                                    },
                                    ErrorPolicy::Retry(ct) => {
                                        // If retry count has been reached, then fail, shutting down the actor
                                        if retry_count >= ct {
                                            break 'event;
                                        }
        
                                        // If not, increment and try again
                                        retry_count += 1;
                                        continue;
                                    },
                                    ErrorPolicy::Shutdown => {
                                        // Shutdown the actor
                                        break 'event;
                                    }
                                }
                            }
                           

                        }
                    }
                    
                }
            }
        }

        // Deinitialize the actor, following deinitialization retry error policy
        {
            let mut retry_count = 0;

            while self.actor.deinitialize(&mut context).await.is_err() {
                // Match on the error policy
                match self.metadata.error_policy.deinitialize {
                    ErrorPolicy::Ignore => {
                        // Ignore the error
                        break;
                    },
                    ErrorPolicy::Retry(ct) => {
                        // If retry count has been reached, then fail, shutting down the actor
                        if retry_count >= ct {
                            // Todo: logging here.
                            return;
                        }

                        // If not, increment and try again
                        retry_count += 1;
                        continue;
                    },
                    // Treat any other as ignore
                    _ => break
                }
            }
        }
    }
}