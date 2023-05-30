use std::any::Any;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use async_trait::async_trait;
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
pub struct ActorContext<E: SystemEvent> {
    /// The actor's metadata
    pub metadata: ActorMetadata,
    /// The system the actor is being called from
    pub system: System<E>,
}

/// This struct contains data surrounding a message
/// If the `request` field is `Some`, then the contained oneshot channel
/// will be called with the `Message::Response`
#[derive(Debug)]
struct MessageData<M: ActorMessage> {
    message: M,
    request: Option<oneshot::Sender<M::Response>>
}

/// The struct which is used to interface with an actor by other actors
#[derive(Clone)]
pub struct ActorHandle<M: ActorMessage> {
    metadata: ActorMetadata,
    message_sender: mpsc::Sender<MessageData<M>>,
}

impl<M: ActorMessage> ActorType for ActorHandle<M> {}

impl<M: ActorMessage + std::fmt::Debug> ActorHandle<M>
    where
    <M as ActorMessage>::Response: std::fmt::Debug {

    pub fn get_metadata(&self) -> ActorMetadata {
        self.metadata.clone()
    }

    pub async fn request(&self, message: M) -> M::Response {

        // Create a oneshot that returns M::Response
        let (tx, rx) = oneshot::channel::<M::Response>();

        // Create the message data
        let message_data = MessageData {
            message,
            request: Some(tx)
        };

        // Send a message with tx and message
        // Todo: Proper error handling
        self.message_sender.send(message_data).await.unwrap();

        // Wait for the response
        rx.await.unwrap()
    }

    pub async fn tell(&self, message: M) {
        // Create the message data
        let message_data = MessageData {
            message,
            request: None
        };

        // Send the message
        // Todo: Proper error handling
        self.message_sender.send(message_data).await.unwrap();
    }
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
    pub fn new(actor: A, id: ActorID, system: System<E>) -> (ActorSupervisor<A,M,E>, ActorHandle<M>) {

        // Create a mpsc channel to send messages through
        let (mtx, mrx) = mpsc::channel(64);

        // Get the broadcast reciever from the system
        let erx = system.event_sender.subscribe();
        
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
        };

        (supervisor, handle)
    }

    pub async fn run(&mut self, system: System<E>) {

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
                            if sender.send(res).is_ok() {
                                // Do this because we really don't care if another actor does nothing with our response
                                // and we want to consume the result so the compiler does not complain.
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
                    // We will default to being tolerant to missed events.
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
pub trait ActorMessage: Clone + Send + Sync + 'static {
    /// The type that the actor will return in response to the message.
    /// Leave as the unit type () if no message is to be returned.
    type Response: Send + Sync + 'static;
}


/// The public-facing Actor trait, which is implemented by the user
#[async_trait]
pub trait Actor<Message: ActorMessage, Event: SystemEvent>: Send + Sync + 'static {


    /// Run when the actor is started
    async fn initialize(&mut self, _context: &mut ActorContext<Event>) {}

    /// Run when an actor is stopped
    async fn deinitialize(&mut self, _context: &mut ActorContext<Event>) {}

    /// Run when the actor recieves a message
    async fn message(&mut self, context: &mut ActorContext<Event>, message: Message) -> Message::Response;

    /// Run when the actor recieves an event
    async fn event(&mut self, context: &mut ActorContext<Event>, event: Event);
}



