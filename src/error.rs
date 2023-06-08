use std::io::Error;

use futures::Future;
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

impl ErrorPolicy {
    /// Handles the error policy for a given async function f. The policy is chosen by calling policy_resolver, passing the error value returned by the function.
    /// If the return value Ok(return), then the policy succeeded. If the contained result is an Error, it should be ignored or logged. If the returned value
    /// is Err(error), then the error should be handled.
    pub async fn handle_policy<T, E, P, F, Fut>(f: F, policy_resolver: P) -> Result<Result<T,E>,E>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        P: Fn(&E) -> ErrorPolicy,
    {
        // Count the number of retries done, in case the policy type is Retry.
        let mut retry_count = 0;

        // Loop until either success or failure
        loop {

            // Call the passed function and save the return type
            let r = f().await;

            // If error, unpack the error. If not, return Ok.
            let Err(returned_error) = r else {
                return Ok(r)
            };

            // Resolve the policy
            let policy = policy_resolver(&returned_error);

            // Match on the policy
            match policy {
                ErrorPolicy::Ignore => {
                    // Ignore the error and return ok
                    return Ok(Err(returned_error));
                },
                ErrorPolicy::Retry(ct) => {
                    // If retry count has been reached, then fail, returning the error
                    if retry_count >= ct {
                        return Err(returned_error)
                    }

                    // If not, increment and try again
                    retry_count += 1;
                    continue;
                },
                ErrorPolicy::Shutdown => {
                    // Fail and return the error
                    return Err(returned_error);
                }
            }
        }
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
    /// The error policy used when the federated message channel is closed
    pub federated_closed: ErrorPolicy,
    /// The error policy used when a federated message handler fials
    pub federated_handler: ErrorPolicy
}

impl Default for ErrorPolicyCollection {
    fn default() -> Self {
        Self {
            initialize: Default::default(),
            deinitialize: Default::default(),
            notify_lag: ErrorPolicy::Ignore,
            notify_closed: ErrorPolicy::Shutdown,
            notify_handler: ErrorPolicy::Ignore,
            federated_closed: ErrorPolicy::Shutdown,
            federated_handler: ErrorPolicy::Ignore
        }
    }
}