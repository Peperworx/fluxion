//! # `ActorSupervisor`
//! The actor supervisor is responsible for handling the actor's entire lifecycle, including dispatching messages
//! and handling shutdowns.

use crate::util::generic_abstractions::{ActorParams, SystemParams};
use crate::Channel;
use crate::{actor::actor_ref::ActorRef, message::MessageHandler};

#[cfg(any(federated, notification))]
use crate::util::generic_abstractions::MessageParams;

#[cfg(serde)]
use crate::message::serializer::MessageSerializer;

#[cfg(foreign)]
use crate::message::foreign::ForeignMessage;

use super::wrapper::ActorWrapper;

#[cfg(foreign)]
use super::Actor;

/// # `SupervisorMessage`
/// An enum that contains different message types depending on feature flags. This is an easy way
/// to send several different types of messages over the same channel.
pub enum SupervisorMessage<AP: ActorParams<S>, S: SystemParams> {
    /// A regular message
    Message(MessageHandler<AP::Message>),
    /// A federated message
    #[cfg(federated)]
    Federated(MessageHandler<<S::SystemMessages as MessageParams>::Federated>),
}

/// # `ActorSupervisor`
/// The struct and task responsible for managing the entire lifecycle of an actor.
pub struct ActorSupervisor<AP: ActorParams<S>, S: SystemParams> {
    /// The wrapped actor wrapper
    actor: ActorWrapper<AP::Actor>,
    /// The message channel responsible for receiving regular messages.
    message_channel: Channel<SupervisorMessage<AP, S>>,
    /// The channel responsible for receiving notifications
    #[cfg(notification)]
    notification_channel: Channel<<S::SystemMessages as MessageParams>::Notification>,
    /// The channel that receives foreign messages
    #[cfg(foreign)]
    foreign_channel: Channel<ForeignMessage>,
}

impl<AP: ActorParams<S>, S: SystemParams> ActorSupervisor<AP, S> {
    /// Creates a new actor supervisor
    pub fn new(
        actor: AP::Actor,
        #[cfg(notification)] notification_channel: Channel<
            <S::SystemMessages as MessageParams>::Notification,
        >,
    ) -> Self {
        // Create the message channel
        let message_channel = Channel::unbounded();

        // Create the foreign channel
        #[cfg(foreign)]
        let foreign_channel = Channel::unbounded();

        // Create the supervisor
        Self {
            actor: ActorWrapper::new(actor),
            message_channel,
            #[cfg(notification)]
            notification_channel,
            #[cfg(foreign)]
            foreign_channel,
        }
    }

    pub fn get_ref(&self) -> ActorRef<AP, S> {
        ActorRef {
            message_sender: self.message_channel.0.clone(),
        }
    }

    /// Executes the actor's main loop
    pub async fn run(&mut self) {
        // Convert the receivers to futures that are easily awaitable
        let mut messages = self.message_channel.1.clone().into_recv_async();

        #[cfg(notification)]
        let mut notifications = self.notification_channel.1.clone().into_recv_async();
        #[cfg(not(notification))]
        let mut notifications = futures::future::pending::<()>();

        #[cfg(foreign)]
        let mut foreign_messages = self.foreign_channel.1.clone().into_recv_async();
        #[cfg(not(foreign))]
        let mut foreign_messages = futures::future::pending::<()>();

        loop {
            futures::select! {
                _notification = notifications => #[cfg(notification)] {
                    // If there is an error, ignore it for nor
                    let Ok(notification) = _notification else {
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
                _foreign = foreign_messages => #[cfg(foreign)] {
                    // If there is an error, ignore it for nor
                    let Ok(mut message) = _foreign else {
                        continue;
                    };

                    // Decode it
                    let decoded = message.decode::<AP, S, S::Serializer, <AP::Actor as Actor>::Error>();

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
                            let res = S::Serializer::serialize::<_, <AP::Actor as Actor>::Error>(res);

                            // If error, continue
                            let Ok(res) = res else {
                                continue;
                            };

                            // Respond
                            let _ = message.respond::<<AP::Actor as Actor>::Error>(res);
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
                            let res = S::Serializer::serialize::<_, <AP::Actor as Actor>::Error>(res);

                            // If error, continue
                            let Ok(res) = res else {
                                continue;
                            };

                            // Respond
                            let _ = message.respond::<<AP::Actor as Actor>::Error>(res);
                        },
                    }
                }
            }
        }
    }
}
