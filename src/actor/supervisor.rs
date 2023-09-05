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
        Self {
            actor: ActorWrapper::new(actor),
            message_channel,
            notification_channel,
            foreign_channel,
        }
    }

    pub fn get_ref(&self) -> ActorRef<M> {
        ActorRef {
            message_sender: self.message_channel.0.clone(),
        }
    }

    /// Executes the actor's main loop
    pub async fn run(&mut self) {
        // Convert the receivers to futures that are easily awaitable
        let mut notifications = self.notification_channel.1.clone().into_recv_async();
        let mut messages = self.message_channel.1.clone().into_recv_async();
        let mut foreign_messages = self.foreign_channel.1.clone().into_recv_async();

        loop {
            futures::select! {
                notification = notifications => {
                    // If there is an error, ignore it for nor
                    let Ok(notification) = notification else {
                        continue;
                    };

                    // Dispatch the notification as a message
                    let _ = self.actor.dispatch(&notification).await;
                },
                message = messages => {
                    // If there is an error, ignore it for nor
                    let Ok(message) = message else {
                        continue;
                    };

                    // Match on the message, and dispatch depending on if it is a regular or federated message
                    match message {
                        SupervisorMessage::Message(mut m) => {
                            // Dispatch
                            let res = self.actor.dispatch(m.message()).await;

                            // If error, continue
                            let Ok(res) = res else {
                                continue;
                            };

                            // Respond
                            let _ = m.respond(res);
                            // Ignore error for now
                        },
                        #[cfg(federated)]
                        SupervisorMessage::Federated(mut m) => {
                            // Dispatch
                            let res = self.actor.dispatch(m.message()).await;

                            // If error, continue
                            let Ok(res) = res else {
                                continue;
                            };

                            // Respond
                            let _ = m.respond(res);
                            // Ignore error for now
                        },
                    };
                },
                foreign = foreign_messages => {
                    // If there is an error, ignore it for nor
                    let Ok(mut message) = foreign else {
                        continue;
                    };

                    // Decode it
                    let decoded = message.decode::<M, G::Serializer, <G::Actor as Actor>::Error>();

                    // If there is an error, ignore it for nor
                    let Ok(decoded) = decoded else {
                        continue;
                    };

                    // match on the type, dispatch, serialize response, and respond
                    match decoded {
                        crate::message::foreign::ForeignType::Message(m) => {
                            // Dispatch
                            let res = self.actor.dispatch(&m).await;

                            // If error, continue
                            let Ok(res) = res else {
                                continue;
                            };

                            // Serialize the response
                            let res = G::Serializer::serialize::<_, <G::Actor as Actor>::Error>(res);

                            // If error, continue
                            let Ok(res) = res else {
                                continue;
                            };

                            // Respond
                            let _ = message.respond::<<G::Actor as Actor>::Error>(res);
                        },
                        #[cfg(federated)]
                        crate::message::foreign::ForeignType::Federated(m) => {
                            // Dispatch
                            let res = self.actor.dispatch(&m).await;

                            // If error, continue
                            let Ok(res) = res else {
                                continue;
                            };

                            // Serialize the response
                            let res = G::Serializer::serialize::<_, <G::Actor as Actor>::Error>(res);

                            // If error, continue
                            let Ok(res) = res else {
                                continue;
                            };

                            // Respond
                            let _ = message.respond::<<G::Actor as Actor>::Error>(res);
                        },
                    }
                }
            }
        }
    }
}
