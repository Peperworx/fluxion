//! The implementation of foreign messages


use tokio::sync::oneshot;

use crate::{actor::path::ActorPath, error::ActorError};

use super::{AsMessageType, Message, MessageType};

#[cfg(not(feature="bincode"))]
use super::DynMessageResponse;
/// # ForeignMessage
/// The enum sent along a foreign message channel.
#[derive(Debug)]
pub enum ForeignMessage<F: Message> {
    /// Contains a federated message sent to a foreign actor
    /// as well as its responder oneshot and target
    FederatedMessage(F, Option<oneshot::Sender<F::Response>>, ActorPath),
    /// Contains a message sent to a foreign actor as well as it's responder and target
    #[cfg(not(feature="bincode"))]
    Message(
        Box<DynMessageResponse>,
        Option<oneshot::Sender<Box<DynMessageResponse>>>,
        ActorPath,
    ),
    #[cfg(feature="bincode")]
    Message(
        Vec<u8>,
        Option<oneshot::Sender<Vec<u8>>>,
        ActorPath,
    ),
}

impl<F: Message> ForeignMessage<F> {
    /// Gets the target of the message
    pub fn get_target(&self) -> &ActorPath {
        match self {
            ForeignMessage::FederatedMessage(_, _, t) => t,
            ForeignMessage::Message(_, _, t) => t,
        }
    }

    /// Pops an actor off of target
    pub fn pop_target(self) -> Self {
        match self {
            ForeignMessage::FederatedMessage(a, b, c) => {
                ForeignMessage::FederatedMessage(a, b, c.popfirst())
            }
            ForeignMessage::Message(a, b, c) => ForeignMessage::Message(a, b, c.popfirst()),
        }
    }
}

impl<F: Message, M: Message> AsMessageType<F, M> for ForeignMessage<F> {
    fn as_message_type(&self) -> Result<MessageType<F, M>, ActorError> {
        Ok(match self {
            ForeignMessage::FederatedMessage(m, _, _) => MessageType::Federated(m.clone()),
            ForeignMessage::Message(m, _, _) => {
                // Downcast
                #[cfg(not(feature="bincode"))]
                let m = m
                    .downcast_ref::<M>()
                    .ok_or(ActorError::ForeignResponseUnexpected)?;

                #[cfg(feature="bincode")]
                #[cfg(not(feature="foreign"))]
                let m = &m;

                #[cfg(feature="bincode")]
                let m: M = bincode::deserialize(m).or(Err(ActorError::ForeignResponseUnexpected))?;

                
                #[cfg(any(not(feature="foreign"), not(feature="bincode")))]
                let res = MessageType::Message(m.clone());
                
                #[cfg(feature="foreign")]
                #[cfg(feature="bincode")]
                let res = MessageType::Message(m);
                res
            }
        })
    }
}

/// # ForeignReciever
/// [`ForeignReciever`] is a trait that can be implemented on any type that can recieve a foreign message.
/// This is used to abstract over actors stored in a system so that it is not necessary to know the type of their messages.
#[cfg(feature = "foreign")]
#[async_trait::async_trait]
pub(crate) trait ForeignReceiver {
    /// The type of the federated message that is sent by both the local and foreign system
    type Federated: Message;

    /// This function recieves a foreign message and handles it
    async fn handle_foreign(
        &self,
        foreign: ForeignMessage<Self::Federated>,
    ) -> Result<(), ActorError>;
}

/// # ForeignMessenger
/// [`ForeignMessenger`] is a trait that can be implemented on any type that is used to send messages to a foreign actor.
#[cfg(feature = "foreign")]
#[async_trait::async_trait]
pub(crate) trait ForeignMessenger {
    /// The type of the federated message that is sent by both the local and foreign system
    type Federated: Message;

    /// This function must be implemented by every [`ForeignMessenger`]. It sends the passed foreign
    /// message to the foreign actor.
    async fn send_raw_foreign(
        &self,
        message: ForeignMessage<Self::Federated>,
    ) -> Result<(), ActorError>;

    /// This function must be implemented by every [`ForeignMessenger]`
    /// It must return true if someone is ready to recieve a foreign message
    async fn can_send_foreign(&self) -> bool;

    /// This function is automatically implemented. It takes a local message, and
    /// sends it using [`ForeignMessenger::send_raw_foreign`]
    async fn send_message_foreign<M: Message>(
        &self,
        message: M,
        responder: Option<oneshot::Sender<M::Response>>,
        target_actor: &ActorPath,
    ) -> Result<(), ActorError> {
        // If we should wait for a response, then do so
        let (foreign_responder, responder_recieve) = if responder.is_some() {
            let channel = oneshot::channel();
            (Some(channel.0), Some(channel.1))
        } else {
            (None, None)
        };

        // Put the message into a foreign message
        let foreign = ForeignMessage::<Self::Federated>::Message(
            {
                #[cfg(not(feature="bincode"))]
                let v = Box::new(message);
                #[cfg(feature="bincode")]
                let v = bincode::serialize(&message).or(Err(ActorError::ForeignSendFail))?;

                v
            },
            foreign_responder,
            target_actor.clone(),
        );

        // If someone is listening for a foreign message, then send the foreign message
        if self.can_send_foreign().await {
            self.send_raw_foreign(foreign).await?;
        } else {
            return Ok(());
        }

        // If we should wait for a response, then do so
        if let (Some(target), Some(source)) = (responder, responder_recieve) {
            // Wait for the foreign response
            let res = source.await.or(Err(ActorError::ForeignRespondFailed))?;

            // Downcast
            #[cfg(not(feature="bincode"))]
            let res = res
                .downcast_ref::<M::Response>()
                .ok_or(ActorError::ForeignResponseUnexpected)?;
            
            #[cfg(feature="bincode")]
            let res: M::Response = bincode::deserialize(&res).or(Err(ActorError::ForeignResponseUnexpected))?;
            
            

            // Relay the response
            #[cfg(all(feature="bincode", feature = "foreign"))]
            target
                .send(res)
                .or(Err(ActorError::ForeignResponseRelayFail))?;
            #[cfg(not(all(feature="bincode", feature = "foreign")))]
            target
                .send(res.clone())
                .or(Err(ActorError::ForeignResponseRelayFail))?;
        }

        Ok(())
    }

    /// This function is automatically implemented. It takes a local federated message, and
    /// sends it using [`ForeignMessenger::send_raw_foreign`]
    async fn send_federated_foreign(
        &self,
        federated: Self::Federated,
        responder: Option<oneshot::Sender<<Self::Federated as Message>::Response>>,
        target_actor: &ActorPath,
    ) -> Result<(), ActorError> {
        // Create the foreign message
        let foreign = ForeignMessage::FederatedMessage(federated, responder, target_actor.clone());

        // Send the foreign message
        self.send_raw_foreign(foreign).await
    }
}
