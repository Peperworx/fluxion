use tokio::sync::{broadcast, mpsc};

use crate::{system::System, message::{Message, Notification, foreign::ForeignMessage, MessageType, handler::HandleNotification, DualMessage}, error::{policy::ErrorPolicy, ActorError}, handle_policy, error_policy};

use super::{handle::ActorHandle, Actor, context::ActorContext};



/// # ActorSupervisor
/// Manages an actor's lifecycle
pub struct ActorSupervisor<A, F: Message, N: Notification, M: Message> {
    /// The actor managed by this supervisor
    actor: A,

    /// The id of this actor
    id: String,

    /// The actor's error policy
    error_policy: SupervisorErrorPolicy,

    /// The notification reciever
    notify: broadcast::Receiver<N>,

    /// The message reciever
    message: mpsc::Receiver<DualMessage<F, M, N>>,

    /// The shutdown reciever
    shutdown: broadcast::Receiver<()>,

    // The system the actor is running on
    //system: System<F, N>,
}

impl<A, F, N, M> ActorSupervisor<A, F, N, M>
where
    A: Actor,
    F: Message,
    N: Notification,
    M: Message {
    /// Creates a new supervisor with the given actor and actor id.
    /// Returns the new supervisor alongside the handle that references this.
    pub fn new(actor: A, id: String, system: &System<F,N>, error_policy: SupervisorErrorPolicy) -> (ActorSupervisor<A, F, N, M>, ActorHandle<F, N, M>) {

        // Create a new message channel
        let (message_sender, message) = mpsc::channel(64);

        // Subscribe to the notification broadcaster
        let notify = system.subscribe_notify();

        // Subscribe to the shutdown reciever
        let shutdown = system.subscribe_shutdown();
        
        // Create the supervisor
        let supervisor = Self {
            actor, notify, message,
            shutdown,
            error_policy,
            id: id.clone(),
        };

        // Create the handle
        let handle = ActorHandle {
            message_sender,
            id,
        };

        // Return both
        (supervisor, handle)
    }
}

impl<A, F, N, M> ActorSupervisor<A, F, N, M>
where
    A: Actor + HandleNotification<N>,
    F: Message,
    N: Notification,
    M: Message {

    


    /// Runs the actor, only returning an error after all error policy options have been exhausted.
    pub async fn run(&mut self) -> Result<(), ActorError> {

        // Create a new actor context for this actor to use
        let mut context = ActorContext;

        // Initialize the actor, following error policy.
        let _ = handle_policy!(
            self.actor.initialize(&mut context).await,
            |_| &self.error_policy.initialize,
            (), ActorError).await?;
        
        // TODO: Log any ignored errors here

        // Begin main loop
        loop {
            // Select on recieving messages
            tokio::select! {
                _ = self.shutdown.recv() => {
                    // Just shutdown no matter what happened
                    break;
                },
                notification = handle_policy!(
                    self.notify.recv().await,
                    |e| match e {
                        &broadcast::error::RecvError::Closed => &self.error_policy.notification_channel_closed,
                        &broadcast::error::RecvError::Lagged(_) => &self.error_policy.notification_channel_lagged,
                    },
                    N, broadcast::error::RecvError) => {
                    

                    // If the policy failed, then exit the loop
                    let Ok(notification) = notification else {
                        break;
                    };

                    // If the policy succeeded, but we failed to recieve, then continue. Otherwise handle it.
                    let Ok(notification) = notification else {
                        continue;
                    };

                    // Call the handler, handling error policy
                    let res = handle_policy!(
                        self.actor.notified(&mut context, notification.clone()).await,
                        |_| &self.error_policy.notification_handler,
                        (), ActorError).await;

                    // If the policy failed, then exit the loop
                    if res.is_err() {
                        break;
                    }
                },
                message = handle_policy!(
                    self.message.recv().await.ok_or(ActorError::MessageChannelClosed),
                    |_| &self.error_policy.message_channel_closed,
                    DualMessage<F, M, N>, ActorError) => {
                    

                    // If the policy failed, then exit the loop
                    let Ok(message) = message else {
                        break;
                    };

                    // If the policy succeeded, but we failed to recieve, then continue. Otherwise handle it.
                    let Ok(message) = message else {
                        continue;
                    };

                    // Get the MessageType, downcasting if foreign
                    let message_type = match message {
                        DualMessage::MessageType(m) => m,
                        DualMessage::ForeignMessage(foreign) => match foreign {
                            ForeignMessage::FederatedMessage(_, _, _) => todo!(),
                            ForeignMessage::Message(_, _, _) => todo!(),
                        },
                    };


                }
            }
        }

        // Deinitialize the actor, following error policy
        let _ = handle_policy!(
            self.actor.deinitialize(&mut context).await,
            |_| &self.error_policy.deinitialize,
            (), ActorError).await?;
        // TODO: Log any ignored errors here

        Ok(())
    }

    /// Cleans up the actor
    pub async fn cleanup(&mut self) -> Result<(), ActorError> {

        // Cleanup the actor, following error policy
        let _ = handle_policy!(
            self.actor.cleanup().await,
            |_| &self.error_policy.cleanup,
            (), ActorError).await?;
        
        // Close the message channel
        self.message.close();

        Ok(())
    }
}



/// # SupervisorErrorPolicy
/// The error policies used by an actor supervisor.
#[derive(Clone, Debug)]
pub struct SupervisorErrorPolicy {
    /// Called when actor initialization fails
    pub initialize: ErrorPolicy<ActorError>,
    /// Called when actor deinitialization fails
    pub deinitialize: ErrorPolicy<ActorError>,
    /// Called when actor cleanup fails
    pub cleanup: ErrorPolicy<ActorError>,
    /// Called when an actor notification channel is dropped.
    /// This should *never* ignore, as it could cause an actor
    /// to be orphaned and run forever.
    pub notification_channel_closed: ErrorPolicy<broadcast::error::RecvError>,
    /// Called when a notification channel laggs
    pub notification_channel_lagged: ErrorPolicy<broadcast::error::RecvError>,
    /// Called when an actor's notification handler fails
    pub notification_handler: ErrorPolicy<ActorError>,
    /// Called when an actor's message channel closes
    pub message_channel_closed: ErrorPolicy<ActorError>,
}

impl Default for SupervisorErrorPolicy {
    fn default() -> Self {
        Self {
            initialize: error_policy! {
                fail;
            },
            deinitialize: error_policy! {
                fail;
            },
            cleanup: error_policy! {
                fail;
            },
            notification_channel_closed: error_policy! {
                fail;
            },
            notification_channel_lagged: error_policy! {
                ignore;
            },
            notification_handler: error_policy! {
                ignore;
            },
            message_channel_closed: error_policy! {
                fail;
            },
        }
    }
}