//! # Error Policy
//! Fluxion defines a very simple Domain Specific Language for defining responses to errors.
//! Error policies are built from commands, which are run every time an error occurs.
//! Error policies may be run in one of three places: when actor initialization fails, when an actor fails to handle a message, and when actor deinitialization or cleanup fail.
//! 

use alloc::vec::Vec;
use alloc::vec;


/// # [`ErrorPolicyCommand`]
/// An [`ErrorPolicyCommand`] represents one step in an [`ErrorPolicy`]
#[derive(Debug, Clone)]
pub enum ErrorPolicyCommand<E> {
    /// Runs the operation
    Run,
    /// Fails the operation, returning the last error
    Fail,
    /// Ignores any potential failure
    Ignore,
    /// Fails the operation if a specific error was encountered
    FailIf(E),
    /// Ignores the failure if a specific error was encountered
    IgnoreIf(E),
    /// Restarts execution of the policy the contained number of times before continuing.
    Loop(usize, usize),
}

/// # [`ErrorPolicy`]
/// [`ErrorPolicy`] dictates how a recieved error is to be handled.
/// An error policy is constructed from a [`Vec`] of [`ErrorPolicyCommand`]s,
/// however users should use the [`crate::error_policy`] macro.
#[derive(Clone, Debug)]
pub struct ErrorPolicy<E>(Option<Vec<ErrorPolicyCommand<E>>>);

impl<E> ErrorPolicy<E> {

    const DEFAULT_POLICY: [ErrorPolicyCommand<E>; 1] = [ErrorPolicyCommand::<E>::Ignore];

    /// Creates a new [`ErrorPolicy`] from a [`Vec<ErrorPolicyCommand<E>>`]
    #[must_use]
    pub fn new(policy: Vec<ErrorPolicyCommand<E>>) -> Self {
        Self(Some(policy))
    }

    /// Returns the contained array of policies
    #[must_use]
    pub fn contained(&mut self) -> Option<&mut [ErrorPolicyCommand<E>]> {
        self.0.as_deref_mut()
    }

    /// Returns true if the default policy is implemented
    #[must_use]
    pub fn is_default(&self) -> bool {
        self.0.is_none()
    }

    /// The default error policy is to ignore all errors.
    #[must_use]
    pub const fn default_policy() -> Self {
        Self(None)
    }
}

impl<E> Default for ErrorPolicy<E> {
    fn default() -> Self {
        Self::default_policy()
    }
}


/// # [`handle_error_policy`]
/// Handles an error policy for a given expression. Returns Ok(Result) if the policy succeeded.
/// If the contained result is Ok, then the operation as a whole succeeded. If the result
/// is an Err, then the operation failed, but the policy succeeded. This can happen if an error is ignored.
/// If an Err is returned, then both the policy and operation failed with the contained error.
///
/// ## Usage
/// ```
/// handle_policy!(
///     checked_expr, policy,
///     return_type, return_error
/// )
/// ```
#[macro_export]
macro_rules! handle_policy {
    ($checked:expr, $policy:expr, $ret:ty, $e:ty) => {
        async {

            // The previous result.
            // Run the operation once.
            let mut prev: Result<$ret, $e> = $checked;

            // If ok, then return.
            let Err(res_err) = &prev else {
                // Weird reassign to allow the return type to be infered
                let res: Result<_, $e> = Ok(prev);
                return res;
            };


            // The position in the policy
            let mut pos = 0;

            // Get the policy
            let mut policy = $policy(res_err);

            // If the default policy was used, ignore
            if policy.is_default() {
                return Ok(prev);
            }

            // Get the internal commands
            // We know this can be safely unwraped because of our
            // previous check
            let coms = policy.contained().unwrap();

            loop {
                // If the operation was a success, then return
                if prev.is_ok() {
                    return Ok(prev);
                }

                // Extract the policy from the option
                let Some(com) = coms.get_mut(pos) else {
                    return Ok(Ok(prev?));
                };

                // Increment pos
                pos += 1;

                // Handle the command
                match com {
                    $crate::error_policy::ErrorPolicyCommand::Run => {
                        // Run the operation
                        prev = $checked;
                        
                    },
                    $crate::error_policy::ErrorPolicyCommand::Fail => {
                        // If we should fail, then just return the error if there is one. This is also the default behavior if there
                        // are no more commands.
                        return Ok(Ok(prev?));
                    },
                    $crate::error_policy::ErrorPolicyCommand::Ignore => {
                        // If we should ignore, then return the result in an Ok
                        return Ok(prev);
                    },
                    $crate::error_policy::ErrorPolicyCommand::FailIf(test) => {
                        // If an error was returned and matches test, then pass it along
                        if prev.as_ref().is_err_and(|e| e == test)  {
                            return Ok(Ok(prev?));
                        }
                    },
                    $crate::error_policy::ErrorPolicyCommand::IgnoreIf(test) => {
                        // If an error was returned and matches test, then ignore the error
                        // and return.
                        if prev.as_ref().is_err_and(|e| e == test)  {
                            return Ok(prev);
                        }
                    },
                    $crate::error_policy::ErrorPolicyCommand::Loop(n, curr) => {
                        // Decrement curr and if it is larger than, 0 return to the beginning
                        *curr -= 1;
                        if *curr > 0 {
                            pos = 0;
                        } else {
                            // Otherwise, reset curr to n
                            *curr = *n;
                        }

                    },

                };
            }
        }
    };
}


/// # [`_error_policy_resolve_single`]
/// Internally used error policy macro to resolve a single policy command.
#[macro_export]
macro_rules! _error_policy_resolve_single {
    (run;) => {
        $crate::error_policy::ErrorPolicyCommand::Run
    };

    (failif $e:expr;) => {
        $crate::error_policy::ErrorPolicyCommand::FailIf($e)
    };

    (ignoreif $e:expr;) => {
        $crate::error_policy::ErrorPolicyCommand::IgnoreIf($e)
    };

    (fail;) => {
        $crate::error_policy::ErrorPolicyCommand::Fail
    };

    (ignore;) => {
        $crate::error_policy::ErrorPolicyCommand::Ignore
    };

    (loop $e:expr;) => {
        $crate::error_policy::ErrorPolicyCommand::Loop($e, $e)
    };
}

/// # [`error_policy`]
/// This macro provides a DSL for defining error policies.
/// Each "statement" is separated by `;`, and has a command name
/// and may have some arguments separated by commas, each of which is an expression.
/// Any command marked terminal will immediately return a success.
/// If marked as always terminal, the command will always return.
/// The commands are as follows:
///
/// ## `run` -- terminal
/// Runs the operation that the error policy handles errors for. Takes no arguments.
/// Example:
/// `run;`
///
/// ## `fail` -- always terminal
/// If the operation has not been run yet, then it runs the operation. Then it directly returns the result returned by the operation,
/// with an Ok(res) if the result was a success, or an Err(e) if not.
///
/// ## `ignore` -- always terminal
/// If the operation has not been run yet, then it runs the operation. Returns a Ok(res) reguardless of the result.
///
/// ## `failif` -- terminal
/// If the operation has not been run yet, then it runs the operation. Fails the same as `fail` if the error is equal to the argument,
/// or continues to the next operation if not.
///
/// ## `ignoreif` -- terminal
/// If the operation has not been run yet, then it runs the operation. Ignores the same as `ignore` if the error is equal to the argument,
/// or continues to the next operation if not.
///
/// ## `loop`
/// Restarts the policy the number of time in the argument. Multiple `loop`s are not supported yet.
///
/// ## Example
/// ```
/// error_policy! {
///     run; // Runs the operation
///     loop 10; // If the operation failed, restart at the beginning 10 times, which will run the operation again
///     failif ActorError::MessageReceiveError; // If the error is an ActorError::MessageReceiveError, then fail. Otherwise continue on.
///     ignoreif ActorError::InvalidMessageType; // If the error is an ActorError::ForeignSendFail, then fail. Otherwise continue.
///     ignore; // Ignore all other errors.
/// }
/// ```
#[macro_export]
macro_rules! error_policy {
    ($command:ident $($arg:expr),*;) => {
        ErrorPolicy::new(vec![$crate::_error_policy_resolve_single!{ $command $($arg),*; }])
    };

    ($command:ident $($arg:expr) *; $($commands:ident $($args:expr) *;)+) => {{
        let mut out = vec![$crate::_error_policy_resolve_single!{ $command $($arg),*; }];
        let cons: Vec<$crate::error::policy::ErrorPolicyCommand<_>> = $crate::error_policy!{ $($commands $($args) *;)+ }.contained().clone();
        out.extend(cons);
        ErrorPolicy::new(out)
    }};
}