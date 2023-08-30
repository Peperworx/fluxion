//! # ActorSupervisor
//! The actor supervisor is responsible for handling the actor's entire lifecycle, including dispatching messages
//! and handling shutdowns.


use alloc::boxed::Box;

use crate::actor::actor_ref::ActorRef;
use crate::message::{Message, Handler};
use crate::error::FluxionError;

#[cfg(serde)]
use {
    crate::message::MessageSerializer,
    alloc::vec::Vec
};

use super::{wrapper::ActorWrapper, Actor, Handle};


pub struct ActorSupervisor<A: Actor> {
    /// The wrapped actor wrapper
    actor: ActorWrapper<A>,
    /// The message channel
    messages: flume::Receiver<Box<dyn Handler<A>>>,
}

/// # Supervisor
/// This trait is implemented by [`ActorSupervisor`] as a way to get around the mess of generics and feature flags by using associated types.
#[cfg_attr(async_trait, async_trait::async_trait)]
pub trait Supervisor {

    /// The actor this supervisor wraps
    type Actor: Actor;

    /// If serde is enabled, this is the struct in charge of serializing messages
    #[cfg(serde)]
    type Serializer: MessageSerializer;

    /// If foreign messages are enabled, this is the message type which the actor will deserialize foreign messages into
    #[cfg(foreign)]
    type Foreign: Message;

    /// If federated messages are enabled, this is the message type fo the federated messages
    #[cfg(federated)]
    type Federated: Message;

    /// If notifications are enabled, this is the message type of the notification
    #[cfg(notification)]
    type Notification: Message;

    /// Create a new Supervisor
    fn new(actor: Self::Actor) -> (Self, ActorRef<Self::Actor>)
    where
        Self: Sized;

    /// Pass a serialized foreign message to the actor, and recieve a serialized response.
    #[cfg(foreign)]
    async fn dispatch_foreign(&mut self, message: Vec<u8>) -> Result<Vec<u8>, FluxionError<<Self::Actor as Actor>::Error>>
    where
        Self::Actor: Handle<Self::Foreign>,
        Self::Foreign: for<'a> serde::Deserialize<'a>,
        <Self::Foreign as Message>::Response: serde::Serialize;
    
    /// Dispatch a regular message
    /// Here we do some fun generic magic to allow actors to implement multiple messages by using an enum that implements From<M>
    /// for different message types, and a response which implementes TryInto for different message types.
    async fn dispatch<R, M, T>(&mut self, message: &T) -> Result<T::Response, FluxionError<<Self::Actor as Actor>::Error>>
    where
        Self::Actor: Handle<M>,
        R: TryInto<T::Response>,
        M: Message<Response = R> + for <'a> From<&'a T>,
        T: Message;

    /// Run the supervisor
    async fn run(&mut self) -> Result<(), FluxionError<<Self::Actor as Actor>::Error>>;
}


#[cfg_attr(async_trait, async_trait::async_trait)]
impl<
    #[cfg(serde)]       S: MessageSerializer,
    #[cfg(serde)]       A: SupervisorActor<Serializer = S>,
    #[cfg(not(serde))]  A: SupervisorActor
> Supervisor for ActorSupervisor<A>
{
    type Actor = A;

    #[cfg(serde)]
    type Serializer = S;

    #[cfg(foreign)]
    type Foreign = A::Foreign;

    /// If federated messages are enabled, this is the message type fo the federated messages
    #[cfg(federated)]
    type Federated = A::Federated;

    /// If notifications are enabled, this is the message type of the notification
    #[cfg(notification)]
    type Notification = A::Notification;

    fn new(actor: Self::Actor) -> (Self, ActorRef<Self::Actor>) {

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
    async fn dispatch_foreign(&mut self, message: Vec<u8>) -> Result<Vec<u8>, FluxionError<<Self::Actor as Actor>::Error>>
    where
        Self::Actor: Handle<Self::Foreign>,
        Self::Foreign: for<'a> serde::Deserialize<'a>,
        <Self::Foreign as Message>::Response: serde::Serialize
    {
        
        // Deserialize the message into Foreign
        let message: Self::Foreign = Self::Serializer::deserialize(message)?;

        // Handle it
        let res = self.actor.dispatch(&message).await?;

        // Reserialize the response and return
        Self::Serializer::serialize(res)
    }


    async fn dispatch<R, M, T>(&mut self, message: &T) -> Result<T::Response, FluxionError<<Self::Actor as Actor>::Error>>
    where
        Self::Actor: Handle<M>,
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


    async fn run(&mut self) -> Result<(), FluxionError<<Self::Actor as Actor>::Error>> {
        loop {
            // Receive the message
            let mut message = self.messages.recv_async().await.unwrap();

            // Retreive a mutable reference to the message
            let message = message.as_mut();
            
            // Run the handler
            message.handle(&mut self.actor).await?;
        }

        Ok(())
    }
}

