use async_trait::async_trait;

use crate::{error::{ActorError, ErrorPolicyCollection}, system::SystemNotification};

use self::context::ActorContext;

/// Contains the ActorContext struct, which defines an actors access to the System.
pub mod context;

/// Contains the ActorSupervisor struct, which runs a task that recieves messages and calls the proper handler.
pub mod supervisor;

/// Contains the ActorHandle struct, which provides an interface with which to interact with an actor
pub mod handle;

/// # ActorID
/// The type by which an actor is identified. Currently set to `String`
pub type ActorID = String;

/// # ActorMetadata
/// Contains the metadata of a specific actor.
#[derive(Clone, Debug)]
pub struct ActorMetadata {
    /// The id of the actor
    id: ActorID,
    /// The error policy collection of the actor
    error_policy: ErrorPolicyCollection,
}

/// # ActorMessage
/// This trait should be implemented for any type which will be used as a message.
/// It contains one associated type `Response`, which should be set to the response type of the message.
pub trait ActorMessage {
    /// The response type of the message
    type Response;
}

/// # Actor
/// The Actor trait is the global trait that all actors must implement. 
/// Actor logic is separated into multiple different traits to decrease the impact of redundant generics. This specific trait contains initialization and deinitialization logic for the actor.
#[async_trait]
pub trait Actor: Send + Sync + 'static {
    /// Called when the actor is started
    async fn initialize(&mut self, context: &mut ActorContext) -> Result<(), ActorError>;

    /// Called when the actor is stopped
    async fn deinitialize(&mut self, context: &mut ActorContext) -> Result<(), ActorError>;
}

/// # NotifyHandler
/// NotifyHandler contains a single function `notified` that is called whenever an actor recieves a notification from the system.
/// The `notified` function has a default implementation that simply returns `Ok(())`.
/// `NotifyHandler<()>` is implemented by default for all implementors of `Actor`, and uses the aformentioned default implementation.
/// This default implementation can be overriden on a per-actor basis.
#[async_trait]
pub trait NotifyHandler<N: SystemNotification> {
    /// Called when the actor recieves a notification
    async fn notified(&mut self, _context: &mut ActorContext, _notification: N) -> Result<(), ActorError> {
        Ok(())
    }
}

#[async_trait]
impl<A> NotifyHandler<()> for A where A: Actor {}

/// # MessageHandler
/// MessageHandler contains a single function `message`, which is called whenever the actor recieves a message.
/// The `message` function should return a `Result` containing either an `Err` or the response type of the message.
/// MessageHandler<M> may be implemented more than once for a single actor, provided that the type `M` is different for each
/// implementation. It is up to the sender of the message to determine which message type `M` to send.
#[async_trait]
pub trait MessageHandler<M: ActorMessage> {
    /// Called when the actor recieves a message
    async fn message(&mut self, context: &mut ActorContext, message: M) -> Result<M::Response, ActorError>;
}

/// # FederatedHandler
/// FederatedHandler contains a single function `federated_message`, which is called whenever the actor recieves a federated message.
/// While FederatedHandler<F> may be implemented multiple times on a single actor for different types `F`, only the handler that matches
/// the current system's federated message type will be called.
#[async_trait]
pub trait FederatedHandler<F: ActorMessage> {
    /// Called when the actor recieves a federated message
    async fn federated_message(&mut self, context: &mut ActorContext, message: F) -> Result<F::Response, ActorError>;
}