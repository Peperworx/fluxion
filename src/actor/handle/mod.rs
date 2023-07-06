//! Contains [`crate::actor::handle::ActorHandle`] (and its various implementations), a trait that is used for interacting with Actors.




use crate::{
    error::ActorError,
    message::Message
};

use super::path::ActorPath;


/// Contains [`crate::actor::handle::local::LocalHandle`], an implementor of [`ActorHandle`] used for communicating with local actors.
pub mod local;


/// Contains [`crate::actor::handle::foreign::ForeignHandle`], an implementor of [`ActorHandle`] used for communicating with foreign actors.
#[cfg(feature = "foreign")]
pub mod foreign;


/// # ActorHandle
/// [`ActorHandle`] is a trait which provides methods through which to interface with actors.
/// This trait exists to allow both Foreign and Local actors to have the same interface.
#[async_trait::async_trait]
pub trait ActorHandle<F, M>: Send + Sync + 'static
where
    F: Message,
    M: Message,
{
    /// Gets the referenced actor's path.
    fn get_path(&self) -> &ActorPath;

    /// Sends a message to the referenced actor and does not wait for a response.
    async fn send(&self, message: M) -> Result<(), ActorError>;

    /// Sends a message to the actor and waits for a response.
    async fn request(&self, message: M) -> Result<M::Response, ActorError>;

    /// Sends a federated message to the referenced actor and does not wait for a response.
    async fn send_federated(&self, message: F) -> Result<(), ActorError>;

    /// Sends a federated message to the referenced actor and waits for a response.
    async fn request_federated(&self, message: F) -> Result<F::Response, ActorError>;
}


