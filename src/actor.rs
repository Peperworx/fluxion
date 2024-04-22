//! # Actors
//! This module contains traits and other types and implementations surrounding actors and how they interface with the system. 


/// # [`Actor`]
/// This trait defines the interface between the system and the actor.
/// The methods defined in this trait provide the actor's only chances to access itself
/// mutably in an async context. 
/// All actors must implement this trait.
pub trait Actor: Send + Sync + 'static {

    /// # [`Error`]
    /// This associated type contains the error type that
    /// can be returned by methods defined by this trait.
    type Error;

    /// # [`initialize`]
    /// Called immediately before the actor is added to the system.
    /// If it fails, it shou
    fn initialize(&mut self) -> impl core::future::Future<Output = Result<(), Self::Error>> + Send {async {
        Ok(())
    }}

    /// # [`deinitialize`]
    /// Called immediately after the actor is shut down.
    /// This will be the last opportunity the actor has to execute any code in an async context.
    fn deinitialize(&mut self) -> impl core::future::Future<Output = Result<(), Self::Error>> + Send {async {
        Ok(())
    }}
}


/// Newtype pattern implementing Slacktor's actor trait
/// for implementorrs of our [`Actor`] trait here.
pub(crate) struct ActorWrapper<T: Actor>(pub T);

impl<R: Actor> slacktor::Actor for ActorWrapper<R> {}