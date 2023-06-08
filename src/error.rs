
use std::io::Error;

use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Clone, Debug, Error)]
pub enum ActorError {
    #[error("The message reiever has ran out of messages")]
    OutOfMessages,
    #[error("A federated message failed to send")]
    FederatedSendError,
}

#[derive(Clone, Debug, Error)]
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
    /// The error policy used when the federated message channel is closed
    pub federated_closed: ErrorPolicy,
    /// The error policy used when a federated message handler fails
    pub federated_handler: ErrorPolicy,
    /// The error policy used when a federated message handler is unable to respond
    pub federated_respond: ErrorPolicy,
    /// The error policy used when a federated message fails to send
    pub federated_send: ErrorPolicy,
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
            federated_handler: ErrorPolicy::Ignore,
            federated_respond: ErrorPolicy::Ignore,
            federated_send: ErrorPolicy::Shutdown,
        }
    }
}

/// Macro to handle error policies. Errors must implement clone.
#[macro_export]

macro_rules! handle_policy {
    ($checked:expr, $policy:expr, $ret:ty, $e:ty) => {
        async {
            // This macro used a bunch of underscores to fix some really weird warnings, without relying on unstable
            // to add attributes to individual expressions.
            let mut _retry_count = 0;

            loop {
                // Get the result
                let res: Result<$ret, $e> = $checked;

                // If failed, then get the error and resolve the policy
                let Err(e) = res else {
                    return Ok(res);
                };

                let _e2 = e.clone();

                // Resolve and handle the policy
                match $policy(_e2) {
                    // If ignoring, just pass through
                    $crate::error::ErrorPolicy::Ignore => {
                        return Ok(Err(e))
                    }
                    $crate::error::ErrorPolicy::Retry(rty) => {
                        if _retry_count >= rty {
                            return Err(e); // If retries reached, then it is a failure
                        } else {
                            _retry_count += 1; // If not, then retry one more time
                        }
                    },
                    $crate::error::ErrorPolicy::Shutdown => {
                        return Err(e);
                    },
                }
            }
        }
    };
}