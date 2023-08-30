use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum FluxionError<E> {
    #[cfg(serde)]
    #[error("error deserializing a foreign message")]
    DeserializeError,
    #[cfg(serde)]
    #[error("error serializing a foreign message")]
    SerializeError,
    #[error("message response failed")]
    ResponseFailed,
    #[error("message failed to send")]
    SendError,
    #[error("error converting between message response types")]
    ResponseConversionError,
    #[error("error from actor")]
    ActorError(E),
}