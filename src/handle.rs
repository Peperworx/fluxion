//! [`LocalHandle`] wraps a communication channel with an actor, allowing messages to be sent to a local actor.
//! [`ActorHandle`] is a trait implemented for structs which send messages to an actor, be it local or foreign.

use core::marker::PhantomData;

use crate::types::{message::{Handler, Message, MessageHandler}, actor::{Actor, ActorId}, Handle, errors::{ActorError, SendError}};



/// # [`ActorHandle`]
/// A trait that provides methods for communicating with an actor using a specific message type
pub trait ActorHandle<M: Message> {

    /// Send a message to the actor, receiving a response back
    async fn send(&self, message: M) -> Result<M::Response, SendError>;

    /// Gets the actor's id
    fn get_id(&self) -> ActorId;
}

/// # [`LocalHandle`]
/// A struct that provides methods for communicating with a local actor
pub struct LocalHandle {
    /// The send side of the message channel
    send: whisk::Channel<Box<dyn Message>>,
    /// The ID of the actor
    id: ActorId,
}



impl<M: Message> ActorHandle<M> for LocalHandle {
    async fn send(&self, message: M) -> Result<<M as Message>::Response, SendError> {
        // Create a message handler
        let mh = MessageHandler::new(message);

        
        todo!()
    }

    fn get_id(&self) -> ActorId {
        todo!()
    }
}