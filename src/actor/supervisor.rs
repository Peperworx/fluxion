//! # `ActorSupervisor`
//! The actor supervisor is responsible for handling the actor's entire lifecycle, including dispatching messages
//! and handling shutdowns.

use crate::message::MessageGenerics;
use crate::Channel;
use crate::{actor::actor_ref::ActorRef, message::MessageHandler};

#[cfg(serde)]
use crate::message::serializer::MessageSerializer;

#[cfg(foreign)]
use alloc::vec::Vec;

#[cfg(foreign)]
use crate::message::foreign::ForeignMessage;

use super::{wrapper::ActorWrapper, Actor, Handle};

/// # `SupervisorGenerics`
/// An absolutely insane method to allow generics passed to an actor supervisor to be controlled by type flags.
/// A single generic is passed, containing a type of [`SupervisorGenerics`]. Then, individually controllable
/// associated types are provided by this trait depending on feature flags.
pub trait SupervisorGenerics<M: MessageGenerics> {
    cfg_if::cfg_if! {
        if #[cfg(all(notification, federated))] {
            /// The actor wrapped by this supervisor
            type Actor: Actor + Handle<M::Message> + Handle<M::Notification> + Handle<M::Federated>;
        } else if #[cfg(notification)] {
            /// The actor wrapped by this supervisor
            type Actor: Actor + Handle<M::Message> + Handle<M::Notification>;
        } else if #[cfg(federated)] {
            /// The actor wrapped by this supervisor
            type Actor: Actor + Handle<M::Message> + Handle<M::Federated>;
        } else {
            /// The actor wrapped by this supervisor
            type Actor: Actor + Handle<<Self::Messages as MessageGenerics>::Message>;
        }
    }

    /// If serde is enabled, this is the struct in charge of serializing messages
    #[cfg(serde)]
    type Serializer: MessageSerializer;
}

/// # `SupervisorMessage`
/// An enum that contains different message types depending on feature flags. This is an easy way
/// to send several different types of messages over the same channel.
pub enum SupervisorMessage<T: MessageGenerics> {
    /// A regular message
    Message(MessageHandler<T::Message>),
    /// A federated message
    Federated(MessageHandler<T::Federated>),
}

/// # `ActorSupervisor`
/// The struct and task responsible for managing the entire lifecycle of an actor.
pub struct ActorSupervisor<G: SupervisorGenerics<M>, M: MessageGenerics> {
    /// The wrapped actor wrapper
    actor: ActorWrapper<G::Actor>,
    /// The message channel responsible for receiving regular messages.
    message_channel: Channel<SupervisorMessage<M>>,
    /// The channel responsible for receiving notifications
    notification_channel: Channel<M::Notification>,
    /// The channel that receives foreign messages
    foreign_channel: Channel<ForeignMessage>,
}

impl<G: SupervisorGenerics<M>, M: MessageGenerics> ActorSupervisor<G, M> {
    /// Creates a new actor supervisor
    pub fn new(actor: G::Actor, notification_channel: Channel<M::Notification>) -> Self {
        // Create the message channel
        let message_channel = Channel::unbounded();

        // Create the foreign channel
        let foreign_channel = Channel::unbounded();

        // Create the supervisor
        let supervisor = Self {
            actor: ActorWrapper::new(actor),
            message_channel,
            notification_channel,
            foreign_channel,
        };

        supervisor
    }

    pub fn get_ref(&self) -> ActorRef<M> {
        ActorRef {
            message_sender: self.message_channel.0.clone(),
        }
    }

    /// Executes the actor's main loop
    pub async fn run(&self) {
        todo!()
    }
}
