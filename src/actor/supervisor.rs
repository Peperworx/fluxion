use tokio::sync::{broadcast, mpsc};

use crate::{system::System, message::{Message, Notification, foreign::ForeignMessage, MessageType}, error::{policy::ErrorPolicy, ActorError}};

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
    message: mpsc::Receiver<MessageType<F, N, M>>,

    /// The foreign reciever
    foreign: mpsc::Receiver<ForeignMessage<F, N>>,

    /// The system the actor is running on
    system: System<F, N>,
}

impl<A: Actor, F: Message, N: Notification, M: Message> ActorSupervisor<A, F, N, M> {
    /// Creates a new supervisor with the given actor and actor id.
    /// Returns the new supervisor alongside the handle that references this.
    pub fn new(actor: A, id: &str, system: System<F,N>, error_policy: SupervisorErrorPolicy) -> (ActorSupervisor<A, F, N, M>, ActorHandle<F, N, M>) {

        // Subscribe to the notification broadcaster
        let notify = system.subscribe_notify();

        // Create a new message channel
        let (message_sender, message) = mpsc::channel(16);

        // Create a new foreign message channel
        let (foreign_sender, foreign) = mpsc::channel(16);

        // Create the supervisor
        let supervisor = Self {
            actor, notify, message, foreign, system,
            error_policy,
            id: id.to_string(),
        };

        // Create the handle
        let handle = ActorHandle {
            message_sender,
            foreign_sender,
            id: id.to_string(),
        };

        // Return both
        (supervisor, handle)
    }


    /// Runs the actor, only returning an error after all error policy options have been exhausted.
    pub async fn run(&mut self) -> Result<(), ActorError> {

        // Create a new actor context for this actor to use
        let mut context = ActorContext;

        

        Ok(())
    }
}



/// # SupervisorErrorPolicy
/// The error policies used by an actor supervisor.
pub struct SupervisorErrorPolicy {
    /// Called when actor initialization fails
    pub initialization_failed: ErrorPolicy<ActorError>,
}

impl Default for SupervisorErrorPolicy {
    fn default() -> Self {
        Self {
            initialization_failed: Default::default()
        }
    }
}