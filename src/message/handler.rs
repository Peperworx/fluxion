//! Message handling traits that can be implemented by actors.

use crate::{
    actor::{context::ActorContext, Actor},
    error::ActorError,
};

use super::{Message, Notification, DefaultFederated, DefaultNotification};

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

/// Automatically implement [`HandleNotification`] for the default notification
#[async_trait::async_trait]
impl<T> HandleNotification<DefaultNotification> for T where T: Actor {
    async fn notified<F: Message>(
        &mut self,
        _context: &mut ActorContext<F, DefaultNotification>,
        _notification: DefaultNotification,
    ) -> Result<(), ActorError> {
        Ok(())
    }
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

/// Automatically implement [`HandleFederated`] for the default federated message
#[async_trait::async_trait]
impl<T> HandleFederated<DefaultFederated> for T where T: Actor {
    async fn federated_message<N: Notification>(
        &mut self,
        _context: &mut ActorContext<DefaultFederated, N>,
        _federated_message: DefaultFederated,
    ) -> Result<(), ActorError> {
        Ok(())
    }
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
