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
pub enum ErrorPolicy {
    /// When this policy is selected, any error will restart the actor, first deinitializing, then jnitializing, then resuming the event loop.
    /// The value contained will be decremented each time the actor is restarted due to the same error. If the value reaches 0, the actor will exit.
    RestartActor(usize),
    /// When this policy is selected, the entire actor will stop and deinitialize will be called when an error is encountered.
    StopActor,
    /// When this policy is selected, the error will be ignored
    Ignore,
}