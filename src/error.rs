use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum FluxionError<E> {
    #[cfg(serde)]
    #[error("error deserializing a foreign message")]
    DeserializeError,
    #[cfg(serde)]
    #[error("error serializing a foreign message")]
    SerializeError,
    #[cfg(serde)]
    #[error("message not in registry")]
    NoRegistryEntry,
    #[error("error from actor")]
    ActorError(E),
}