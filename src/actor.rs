use std::any::Any;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use crate::system::{System, SystemEvent};

/// This is defined separate from the Actor trait to allow us to include it in the
/// hashmap inside of System
pub(crate) trait ActorType: Any + Send + Sync + 'static {}

/// An actor's ID. For now this is a String
pub type ActorID = String;



/// Contains an actor's metadata
#[derive(Clone)]
pub struct ActorMetadata {
    /// The actor's ID
    pub id: ActorID,
}

/// Contains information passed to an actor when it is called into.
/// This provides the actor with both its metadata and system
#[derive(Clone)]
pub struct ActorContext {
    /// The actor's metadata
    pub metadata: ActorMetadata,
    /// The system the actor is being called from
    pub system: System,
}

/// This struct contains data surrounding a message
/// If the `request` field is `Some`, then the contained oneshot channel
/// will be called with the `Message::Response`
struct MessageData<M: ActorMessage> {
    message: M,
    request: Option<oneshot::Sender<M::Response>>
}

/// The struct which is used to interface with an actor by other actors
pub struct ActorHandle<M: ActorMessage, E: SystemEvent> {
    metadata: ActorMetadata,
    message_sender: mpsc::Sender<MessageData<M>>,
    event_sender: broadcast::Sender<E>
}

/// The struct in charge of handling an actor's lifecycle.
/// This struct's `run` method is run asynchronously in a separate task
pub struct ActorSupervisor<A: Actor<M, E>, M: ActorMessage, E: SystemEvent> {
    metadata: ActorMetadata,
    actor: A,
    message_reciever: mpsc::Receiver<MessageData<M>>,
    event_reciever: broadcast::Receiver<E>
}

impl<A: Actor<M, E>, M: ActorMessage, E: SystemEvent> ActorSupervisor<A,M,E> {
    pub fn new(actor: A, id: ActorID) -> (ActorSupervisor<A,M,E>, ActorHandle<M, E>) {

        // Create a mpsc channel to send messages through
        let (mtx, mrx) = mpsc::channel(64);

        // Create a broadcast channel to send events through
        let (etx, erx) = broadcast::channel(64);

        
        // Create the supervisor
        let supervisor = ActorSupervisor {
            metadata: ActorMetadata { id: id.clone() },
            actor,
            message_reciever: mrx,
            event_reciever: erx
        };

        // Create the handle
        let handle = ActorHandle {
            metadata: ActorMetadata { id },
            message_sender: mtx,
            event_sender: etx,
        };

        (supervisor, handle)
    }

    pub async fn run(&mut self, system: System) {

        // Create an actor context
        let mut context = ActorContext {
            metadata: self.metadata.clone(),
            system: system.clone(),
        };

        // Initialize the actor
        self.actor.initialize(&mut context).await;

        

        // Continuously recieve messages
        loop {
            tokio::select! {
                message = self.message_reciever.recv() => {
                    if let Some(m) = message {

                        let res = self.actor.message(&mut context, m.message).await;

                        if let Some(sender) = m.request {
                            match sender.send(res) {
                                Ok(_) => {},
                                Err(_) => {} // Do this because we really don't care if another actor does nothing with our response
                            }
                        }
                    } else {
                        // If None is recieved from the message reciever, it means
                        // that all references to the actor's sender have been exhausted.
                        // This can only ever happen if the actor is removed from the system.
                        // (The system maintains an ActorHandle in its hashmap), so if this happens
                        // we should cleanup the actor.
                        break;
                    }
                },
                event = self.event_reciever.recv() => {
                    if let Ok(e) = event {
                        self.actor.event(&mut context, e).await;
                    }
                }
            }
        }
        // Deinitialize the actor
        self.actor.deinitialize(&mut context).await;

        // Cleanup the message reciever
        self.message_reciever.close();
    }
}

/// Represents a message that can be sent to a specific actor
pub trait ActorMessage {
    /// The type that the actor will return in response to the message.
    /// Leave as the unit type () if no message is to be returned.
    type Response;
}


/// The public-facing Actor trait, which is implemented by the user
pub trait Actor<Message: ActorMessage, Event: SystemEvent> {


    /// Run when the actor is started
    async fn initialize(&mut self, context: &mut ActorContext) {}

    /// Run when an actor is stopped
    async fn deinitialize(&mut self, context: &mut ActorContext) {}

    /// Run when the actor recieves a message
    async fn message(&mut self, context: &mut ActorContext, message: Message) -> Message::Response;

    /// Run when the actor recieves an event
    async fn event(&mut self, context: &mut ActorContext, event: Event);
}



