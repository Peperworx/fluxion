



/// # ErrorPolicyCommand
/// An [`ErrorPolicyCommand`] contains one step in an [`ErrorPolicy`]
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
    Loop(usize),
}

/// # ErrorPolicy
/// [`ErrorPolicy`] dictates how a recieved error is to be handled.
/// An error policy is constructed from a [`Vec`] of [`ErrorPolicyCommand`]s,
/// however users should use the [`error_policy`] macro.
#[derive(Clone, Debug)]
pub struct ErrorPolicy<E: Clone>(Vec<ErrorPolicyCommand<E>>);

impl<E: Clone> ErrorPolicy<E> {
    pub fn new(policy: Vec<ErrorPolicyCommand<E>>) -> Self {
        Self(policy)
    }

    /// Returns the contained vec of policies
    pub fn contained(&self) -> &Vec<ErrorPolicyCommand<E>> {
        &self.0
    }

}

impl<E: Clone> Default for ErrorPolicy<E> {
    fn default() -> Self {
        crate::error_policy! {
            run;
            loop 5;
            fail;
        }
    }
}

/// # handle_error_policy
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
            if prev.is_ok() {
                // Weird reassign to allow the return type to be infered
                let res: Result<_, $e> = Ok(prev);
                return res;
            }

            // The number of iterations in the current loop
            let mut loops = 0;

            // The position in the policy
            let mut pos = 0;

            loop {
                // If at any point the operation was a success, then return
                let Err(res_err) = &prev else {
                    return Ok(prev);
                };

                // Get the policy
                let policy = $policy(res_err);

                // Get the internal commands
                let com = policy.contained();

                // Extract the policy from the option
                let Some(com) = com.get(pos) else {
                    return Ok(Ok(prev?));
                };

                // Increment pos
                pos += 1;

                // Handle the command
                match com {
                    $crate::error::policy::ErrorPolicyCommand::Run => {
                        // Run the operation, updating the previous value
                        prev = $checked;
                    },
                    $crate::error::policy::ErrorPolicyCommand::Fail => {
                        // If we should fail, then just return the error if there is one. This is also the default behavior if there
                        // are no more commands.
                        return Ok(Ok(prev?));
                    },
                    $crate::error::policy::ErrorPolicyCommand::Ignore => {
                        // If we should ignore, then return the result in an Ok
                        return Ok(prev);
                    },
                    $crate::error::policy::ErrorPolicyCommand::FailIf(test) => {
                        // If the operation was a success, then return
                        if prev.is_ok() {
                            return Ok(prev);
                        }

                        // If an error was returned and matches test, then pass it along
                        if prev.as_ref().is_err_and(|e| e == test)  {
                            return Ok(Ok(prev?));
                        }
                    },
                    $crate::error::policy::ErrorPolicyCommand::IgnoreIf(test) => {
                        
                        // If an error was returned and matches test, then ignore the error
                        // and return.
                        if prev.as_ref().is_err_and(|e| e == test)  {
                            return Ok(prev);
                        }
                    },
                    $crate::error::policy::ErrorPolicyCommand::Loop(n) => {
                        // Increment loops
                        loops += 1;

                        // If loops is smaller than or equal to n, then reset pos to 0.
                        if &loops < n {
                            pos = 0;
                        } else {
                            // Otherwise, skip this value and set loops to zero
                            loops = 0;
                        }
                    },

                };
            }
        }
    };
}


/// # _error_policy_resolve_single
/// Internally used error policy macro to resolve a single policy command.
#[macro_export]
macro_rules! _error_policy_resolve_single {

    (run;) => {
        $crate::error::policy::ErrorPolicyCommand::Run
    };

    (failif $e:expr;) => {
        $crate::error::policy::ErrorPolicyCommand::FailIf($e)
    };

    (ignoreif $e:expr;) => {
        $crate::error::policy::ErrorPolicyCommand::IgnoreIf($e)
    };

    (fail;) => {
        $crate::error::policy::ErrorPolicyCommand::Fail
    };

    (ignore;) => {
        $crate::error::policy::ErrorPolicyCommand::Ignore
    };

    (loop $e:expr;) => {
        $crate::error::policy::ErrorPolicyCommand::Loop($e)
    }

}    

/// # error_policy
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
///     failif ActorError::ForeignRespondFail; // If the error is an ActorError::ForeignRespondFail, then fail. Otherwise continue on.
///     ignoreif ActorError::ForeignSendFail; // If the error is an ActorError::ForeignSendFail, then fail. Otherwise continue.
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
