use tokio::sync::{broadcast, mpsc};

use crate::{
    error::{policy::ErrorPolicy, ActorError},
    error_policy, handle_policy,
    message::{
        foreign::ForeignMessage,
        handler::{HandleFederated, HandleMessage, HandleNotification},
        AsMessageType, DualMessage, LocalMessage, Message, MessageType, Notification,
    },
    system::System,
};

use super::{context::ActorContext, handle::LocalHandle, path::ActorPath, Actor};

/// # ActorSupervisor
/// Manages an actor's lifecycle
pub struct ActorSupervisor<A, F: Message, N: Notification, M: Message> {
    /// The actor managed by this supervisor
    actor: A,

    /// This actor's path
    path: ActorPath,

    /// The actor's error policy
    error_policy: SupervisorErrorPolicy,

    /// The notification reciever
    notify: broadcast::Receiver<N>,

    /// The message reciever
    message: mpsc::Receiver<DualMessage<F, M>>,

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
    M: Message,
{
    /// Creates a new supervisor with the given actor and actor id.
    /// Returns the new supervisor alongside the handle that references this.
    pub fn new(
        actor: A,
        path: ActorPath,
        system: &System<F, N>,
        error_policy: SupervisorErrorPolicy,
    ) -> (ActorSupervisor<A, F, N, M>, LocalHandle<F, M>) {
        // Create a new message channel
        let (message_sender, message) = mpsc::channel(64);

        // Subscribe to the notification broadcaster
        let notify = system.subscribe_notify();

        // Subscribe to the shutdown reciever
        let shutdown = system.subscribe_shutdown();

        // Create the supervisor
        let supervisor = Self {
            actor,
            notify,
            message,
            shutdown,
            error_policy,
            path: path.clone(),
        };

        // Create the handle
        let handle = LocalHandle {
            message_sender,
            path,
        };

        // Return both
        (supervisor, handle)
    }
}

impl<A, F, N, M> ActorSupervisor<A, F, N, M>
where
    A: Actor + HandleNotification<N> + HandleFederated<F> + HandleMessage<M>,
    F: Message,
    N: Notification,
    M: Message,
{
    /// Runs the actor, only returning an error after all error policy options have been exhausted.
    pub async fn run(&mut self, system: System<F, N>) -> Result<(), ActorError> {
        // Create a new actor context for this actor to use
        let mut context = ActorContext {
            path: self.path.clone(),
            system,
        };

        // Initialize the actor, following error policy.
        let _ = handle_policy!(
            self.actor.initialize(&mut context).await,
            |_| &self.error_policy.initialize,
            (),
            ActorError
        )
        .await?;

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
                    |e: &broadcast::error::RecvError | match e {
                        broadcast::error::RecvError::Closed => &self.error_policy.notification_channel_closed,
                        broadcast::error::RecvError::Lagged(_) => &self.error_policy.notification_channel_lagged,
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
                    DualMessage<F, M>, ActorError) => {


                    // If the policy failed, then exit the loop
                    let Ok(message) = message else {
                        break;
                    };

                    // If the policy succeeded, but we failed to recieve, then continue. Otherwise handle it.
                    let Ok(message) = message else {
                        continue;
                    };

                    // Get the message, downcasting if foreign. This always does at least one clone, but it appears to be unavoidable.
                    let message_type = handle_policy!(
                        message.as_message_type(),
                        |_| &self.error_policy.unexpected_foreign,
                        MessageType<F, M>, ActorError).await;

                    // If the policy failed, exit
                    let Ok(message_type) = message_type else {
                        break;
                    };

                    // If the policy succeeded, but we failed to recieve, then continue.
                    let Ok(message_type) = message_type else {
                        continue;
                    };

                    // Given the message type, call the proper handler
                    match message_type {
                        MessageType::Federated(m) => {
                            let res = handle_policy!(
                                self.actor.federated_message(&mut context, m.clone()).await,
                                |_| &self.error_policy.federated_handler,
                                F::Response, ActorError).await;

                            // If the policy failed, exit
                            let Ok(res) = res else {
                                break;
                            };

                            // If the policy succeeded, but we failed to recieve, then continue.
                            let Ok(res) = res else {
                                continue;
                            };

                            // Match on the responder
                            let responder = match message {
                                DualMessage::LocalMessage(LocalMessage::Federated(_, Some(responder))) => Some(responder),
                                DualMessage::ForeignMessage(ForeignMessage::FederatedMessage(_, Some(responder), _)) => Some(responder),
                                _ => None
                            };

                            // If we need to respond, do so
                            if let Some(responder) = responder {
                                // This is a oneshot, so ignore if error
                                let _ = responder.send(res);
                            }
                        },
                        MessageType::Message(m) => {
                            let res = handle_policy!(
                                self.actor.message(&mut context, m.clone()).await,
                                |_| &self.error_policy.message_handler,
                                M::Response, ActorError).await;

                            // If the policy failed, exit
                            let Ok(res) = res else {
                                break;
                            };

                            // If the policy succeeded, but we failed to recieve, then continue.
                            let Ok(res) = res else {
                                continue;
                            };

                            // Match on the responder, and respond if found
                            match message {
                                DualMessage::LocalMessage(LocalMessage::Message(_, Some(responder))) => {
                                    // Just send the response, ignoring the error
                                    let _ = responder.send(res);
                                },
                                DualMessage::ForeignMessage(ForeignMessage::Message(_, Some(responder), _)) => {
                                    // Box and send the response
                                    let _ = responder.send(Box::new(res));
                                },
                                _ => {}
                            };
                        },
                    };
                }
            }
        }

        // Deinitialize the actor, following error policy
        let _ = handle_policy!(
            self.actor.deinitialize(&mut context).await,
            |_| &self.error_policy.deinitialize,
            (),
            ActorError
        )
        .await?;
        // TODO: Log any ignored errors here

        Ok(())
    }

    /// Cleans up the actor
    pub async fn cleanup(&mut self) -> Result<(), ActorError> {
        // Cleanup the actor, following error policy
        let _ = handle_policy!(
            self.actor.cleanup().await,
            |_| &self.error_policy.cleanup,
            (),
            ActorError
        )
        .await?;

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
    /// Called when a foreign message failed to downcast
    pub unexpected_foreign: ErrorPolicy<ActorError>,
    /// Called when an actor's federated message handler fails
    pub federated_handler: ErrorPolicy<ActorError>,
    /// Called when an actor's message handler fails
    pub message_handler: ErrorPolicy<ActorError>,
    /// Called when a federated message fails to send its response
    pub federated_respond: ErrorPolicy<ActorError>,
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
            unexpected_foreign: error_policy! {
                ignore;
            },
            federated_handler: error_policy! {
                ignore;
            },
            message_handler: error_policy! {
                ignore;
            },
            federated_respond: error_policy! {
                ignore;
            },
        }
    }
}
