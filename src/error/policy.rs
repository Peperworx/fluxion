



/// # ErrorPolicyCommand
/// An [`ErrorPolicyCommand`] contains one step in an [`ErrorPolicy`]
#[derive(Debug)]
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
#[derive(Debug)]
pub struct ErrorPolicy<E>(Vec<ErrorPolicyCommand<E>>);

impl<E> ErrorPolicy<E> {
    pub fn new(policy: Vec<ErrorPolicyCommand<E>>) -> Self {
        Self(policy)
    }

    /// Returns the contained vec of policies
    pub fn contained(self) -> Vec<ErrorPolicyCommand<E>> {
        self.0
    }

    /// Handles an error policy for a given function F. Returns Ok(Result) if the policy succeeded.
    /// If the contained result is Ok, then the operation as a whole succeeded. If the result
    /// is an Err, then the operation failed, but the policy succeeded. This can happen if an error is ignored.
    /// If an Err is returned, then both the policy and operation failed with the contained error.
    pub async fn handle<F, Fut, T>(&self, f: F) -> Result<Result<T, E>, E>
    where
        F: Fn() -> Fut,
        Fut: futures::Future<Output = Result<T, E>>,
        E: PartialEq
    {

        // The number of iterations in the current loop
        let mut loops = 0;

        // The position in the policy
        let mut pos = 0;

        // The previous result
        let mut prev: Option<Result<T, E>> = None;

        loop {
            // Get the command
            let Some(com) = self.0.get(pos) else {
                // If no command, pass along the result, returning an Err if error, and Ok(Ok) if Ok
                let res = if let Some(res) = prev {
                    res
                } else {
                    // Otherwise, call the function
                    f().await
                };
                
                return Ok(Ok(res?));
            };

            // Increment pos
            pos += 1;
            
            // Handle the command
            match com {
                ErrorPolicyCommand::Run => {
                    // Run the operation, updating the previous value
                    prev = Some(f().await);
                },
                ErrorPolicyCommand::Fail => {
                    // If we should fail, then just return the error if there is one. This is also the default behavior if there
                    // are no more commands.
                    let res = if let Some(res) = prev {
                        res
                    } else {
                        // Otherwise, call the function
                        f().await
                    };
                    return Ok(Ok(res?));
                },
                ErrorPolicyCommand::Ignore => {
                    // If we should ignore, then return the result in an Ok
                    let res = if let Some(res) = prev {
                        res
                    } else {
                        // Otherwise, call the function
                        f().await
                    };
                    return Ok(res);
                },
                ErrorPolicyCommand::FailIf(test) => {
                    // Evaluate the previous value
                    let res = if let Some(res) = prev {
                        res
                    } else {
                        // Otherwise, call the function
                        f().await
                    };

                    // If it is a success, then return
                    if res.is_ok() {
                        return Ok(res);
                    }

                    // If an error was returned and matches test, then pass it along
                    if res.as_ref().is_err_and(|e| e == test)  {
                        return Ok(Ok(res?));
                    }

                    // Update the previous value
                    prev = Some(res);
                },
                ErrorPolicyCommand::IgnoreIf(test) => {
                    // Evaluate the previous value
                    let res = if let Some(res) = prev {
                        res
                    } else {
                        // Otherwise, call the function
                        f().await
                    };

                    // If it is a success, then return
                    if res.is_ok() {
                        return Ok(res);
                    }

                    // If an error was returned and matches test, then ignore the error
                    // and return.
                    if res.as_ref().is_err_and(|e| e == test)  {
                        return Ok(res);
                    }
                    
                    // Update the previous value
                    prev = Some(res);
                },
                ErrorPolicyCommand::Loop(n) => {
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
}

impl<E> Default for ErrorPolicy<E> {
    fn default() -> Self {
        crate::error_policy! {
            run;
            loop 5;
            fail;
        }
    }
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
        out.extend($crate::error_policy!{ $($commands $($args) *;)+ }.contained());
        ErrorPolicy::new(out)
    }};
}
