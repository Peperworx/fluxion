//! Message handling traits that can be implemented by actors.

use crate::{
    actor::{context::ActorContext, Actor},
    error::ActorError,
};

use super::{Message, Notification};

/// # HandleNotification
/// This trait implements a single function, [`Self::notified`], which is called whenever
/// the actor recieves a notification.
#[async_trait::async_trait]
pub trait HandleNotification<N: Notification>: Actor {
    /// Called when the actor recieves a notification
    async fn notified<F: Message>(
        &mut self,
        context: &mut ActorContext<F, N>,
        notification: N,
    ) -> Result<(), ActorError>;
}

/// # HandleFederated
/// This trait implements a single function, [`Self::federated_message`], which is called
/// whenever the actor recieves a federated message.
#[async_trait::async_trait]
pub trait HandleFederated<F: Message>: Actor {
    /// Called when the actor recieves a federated message
    async fn federated_message<N: Notification>(
        &mut self,
        context: &mut ActorContext<F, N>,
        federated_message: F,
    ) -> Result<F::Response, ActorError>;
}

/// # HandleMessage
/// This trait implements a single function, [`Self::message`], which is called whenever
/// the actor recieves a message.
#[async_trait::async_trait]
pub trait HandleMessage<M: Message>: Actor {
    /// Called when the actor recieves a message
    async fn message<F: Message, N: Notification>(
        &mut self,
        context: &mut ActorContext<F, N>,
        message: M,
    ) -> Result<M::Response, ActorError>;
}
