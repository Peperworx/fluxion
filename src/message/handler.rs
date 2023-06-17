//! Message handling traits that can be implemented by actors.

use crate::{
    error::ActorError,
    actor::{Actor, context::ActorContext}
};




/// # HandleNotification
/// This trait implements a single function, [`Self::notified`], which is called whenever
/// the actor recieves a notification.
#[async_trait::async_trait]
pub trait HandleNotification<N>: Actor {
    /// Called when the actor recieves a notification
    async fn notified(&mut self, context: &mut ActorContext, notification: &N) -> Result<(), ActorError>;
}

/// # HandleFederated
/// This trait implements a single function, [`Self::federated_message`], which is called
/// whenever the actor recieves a federated message.
#[async_trait::async_trait]
pub trait HandleFederated<F>: Actor {
    /// Called when the actor recieves a federated message
    async fn federated_message(&mut self, context: &mut ActorContext, federated_message: F) -> Result<(), ActorError>;
}

/// # HandleMessage
/// This trait implements a single function, [`Self::message`], which is called whenever
/// the actor recieves a message. 
#[async_trait::async_trait]
pub trait HandleMessage<M>: Actor {
    /// Called when the actor recieves a message
    async fn message(&mut self, context: &mut ActorContext, message: M) -> Result<(), ActorError>;
}