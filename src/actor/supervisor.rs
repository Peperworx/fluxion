//! # ActorSupervisor
//! The actor supervisor is responsible for handling the actor's entire lifecycle, including dispatching messages
//! and handling shutdowns.


use alloc::boxed::Box;

use crate::actor::actor_ref::ActorRef;
use crate::message::{Message, Handler};
use crate::error::FluxionError;

#[cfg(serde)]
use crate::message::MessageSerializer;

#[cfg(foreign)]
use alloc::vec::Vec;

use super::{wrapper::ActorWrapper, Actor, Handle};




/// # SupervisorGenerics
/// An absolutely insane method to allow generics passed to an actor supervisor to be controlled by type flags.
/// A single generic is passed, containing a type of [`SupervisorGenerics`]. Then, individually controllable
/// associated types are provided by this trait depending on feature flags.
pub trait SupervisorGenerics {

    /// The actor wrapped by this supervisor
    type Actor: Actor;

    /// If serde is enabled, this is the struct in charge of serializing messages
    #[cfg(serde)]
    type Serializer: MessageSerializer;

    /// If foreign messages are enabled, this is the message type which the actor will deserialize foreign messages into
    #[cfg(foreign)]
    type Foreign: Message;

    /// If notifications are enabled, this is the message type of the notification
    #[cfg(notification)]
    type Notification: Message;
}




pub struct ActorSupervisor<G: SupervisorGenerics> {
    /// The wrapped actor wrapper
    actor: ActorWrapper<G::Actor>,
    /// The message channel
    messages: flume::Receiver<Box<dyn Handler<G::Actor>>>,
}

impl<G: SupervisorGenerics> ActorSupervisor<G> {
    pub fn new(actor: G::Actor) -> (Self, ActorRef<G::Actor>) {

        // Create the message channel
        let (message_sender, messages) = flume::unbounded();

        // Create the supervisor
        let supervisor = Self {
            actor: ActorWrapper::new(actor),
            messages
        };

        // Create the actor reference
        let reference = ActorRef {
            message_sender
        };

        (supervisor, reference)
    }

    #[cfg(foreign)]
    pub async fn dispatch_foreign(&mut self, message: Vec<u8>) -> Result<Vec<u8>, FluxionError<<G::Actor as Actor>::Error>>
    where
        G::Actor: Handle<G::Foreign>,
        G::Foreign: for<'a> serde::Deserialize<'a>,
        <G::Foreign as Message>::Response: serde::Serialize
    {
        
        // Deserialize the message into Foreign
        let message: G::Foreign = G::Serializer::deserialize(message)?;

        // Handle it
        let res = self.actor.dispatch(&message).await?;

        // Reserialize the response and return
        G::Serializer::serialize(res)
    }


    pub async fn dispatch<R, M, T>(&mut self, message: &T) -> Result<T::Response, FluxionError<<G::Actor as Actor>::Error>>
    where
        G::Actor: Handle<M>,
        R: TryInto<T::Response>,
        M: Message<Response = R> + for <'a> From<&'a T>,
        T: Message
    {
        // Convert the message
        let message = M::from(message);

        // Handle the message
        let res = self.actor.dispatch(&message).await?;

        // Try to convert the response
        res.try_into().or(Err(FluxionError::ResponseConversionError))
    }


    pub async fn run(&mut self) -> Result<(), FluxionError<<G::Actor as Actor>::Error>> {
        loop {
            // Receive the message
            let mut message = self.messages.recv_async().await.unwrap();

            // Retreive a mutable reference to the message
            let message = message.as_mut();
            
            // Run the handler
            message.handle(&mut self.actor).await?;
        }

        
    }
}