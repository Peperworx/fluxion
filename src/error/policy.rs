
/// # ErrorPolicyCommand
/// An [`ErrorPolicyCommand`] contains one step in an [`ErrorPolicy`]
#[derive(Debug)]
pub enum ErrorPolicyCommand<E> {
    /// Continues to the next step if there was an error,
    /// returns if a success
    Pass,
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
pub struct ErrorPolicy<E>(Vec<ErrorPolicyCommand<E>>);

impl<E> ErrorPolicy<E> {
    pub fn new(policy: Vec<ErrorPolicyCommand<E>>) -> Self {
        Self(policy)
    }
}


/// # _error_policy_resolve_single
/// Internally used error policy macro to resolve a single policy command.
#[macro_export]
macro_rules! _error_policy_resolve_single {
    (pass;) => {
        $crate::error::policy::ErrorPolicyCommand::Pass
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
#[macro_export]
macro_rules! error_policy {
    ($command:ident $($arg:expr) *;) => {
        vec![$crate::_error_policy_resolve_single!{ $command $($arg) *; }]
    };

    ($command:ident $($arg:expr) *; $($commands:ident $($args:expr) *;)+) => {{
        let mut out = vec![$crate::_error_policy_resolve_single!{ $command $($arg) *; }];
        out.extend(error_policy!{ $($commands $($args) *;)+ });
        out
    }};
}
