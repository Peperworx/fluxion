use thiserror_no_std::Error;

#[derive(Error)]
pub enum FluxionError<E> {
    #[cfg(serde)]
    #[error("error deserializing a foreign message")]
    DeserializeError,
    #[error("error from actor")]
    ActorError(E),
}