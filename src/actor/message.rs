use tokio::sync::oneshot;

use crate::error::ActorError;

/// # ActorMessage
/// This trait should be implemented for any type which will be used as a message.
/// It contains one associated type `Response`, which should be set to the response type of the message.
pub trait ActorMessage: Clone + Send + Sync + 'static {
    /// The response type of the message
    type Response: Clone + Send + Sync + 'static;
}

/// # MessageType
/// This enum provides different message types that actors can recieve.
pub enum MessageType<F: ActorMessage> {
    /// A Federated Message. If `1` is Some, then a response is expected.
    FederatedMessage(F, Option<oneshot::Sender<Result<F::Response, ActorError>>>),
    // A regular message. If `1` is Some, then a response is expected.
    //Message(M, Option<oneshot::Sender<Result<M::Response, ActorError>>>),
}