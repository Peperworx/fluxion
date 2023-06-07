use thiserror::Error;

#[derive(Debug, Error)]
pub enum ActorError {

}

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("The actor with that id already exists in the system")]
    ActorAlreadyExists,
}

/// # ErrorPolicy
/// Dictates a policy to be followed when an error is encountered
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ErrorPolicy {
    /// When this policy is selected, the error will be ignored
    Ignore,
    /// When this policy is selected, the operation will be retried until a number of attempts equal to the contents of this variant have been reached
    Retry(usize),
    /// When this policy is selected, the actor's main loop is broken and the actor is shutdown. This can only happen in the actor's main loop.
    Shutdown,
}

impl Default for ErrorPolicy {
    fn default() -> Self {
        Self::Retry(10)
    }
}


/// # ErrorPolicyCollection
/// Contains a collection of error policies for different situations.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ErrorPolicyCollection {
    /// The error policy used when initialization fails
    pub initialize: ErrorPolicy,
    /// The error policy used when deinitialization fails
    pub deinitialize: ErrorPolicy,
    /// The error policy used when a notification lags
    pub notify_lag: ErrorPolicy,
    /// The error policy used when a notification channel is closed
    pub notify_closed: ErrorPolicy,
    /// The error policy used when a notification handler fails
    pub notify_handler: ErrorPolicy,
}

impl Default for ErrorPolicyCollection {
    fn default() -> Self {
        Self {
            initialize: Default::default(),
            deinitialize: Default::default(),
            notify_lag: ErrorPolicy::Ignore,
            notify_closed: ErrorPolicy::Shutdown,
            notify_handler: ErrorPolicy::Ignore
        }
    }
}