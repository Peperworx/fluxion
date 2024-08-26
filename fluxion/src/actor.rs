//! # Actors
//! This module contains traits and other types and implementations surrounding actors and how they interface with the system. 

use alloc::sync::Arc;

use crate::{Delegate, Fluxion, Message};



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
    fn initialize(&mut self) -> impl core::future::Future<Output = Result<(), Self::Error>> + Send {async {
        Ok(())
    }}

    /// # [`deinitialize`]
    /// Called immediately after the actor is shut down.
    /// This will be the last opportunity the actor has to execute any code in an async context.
    fn deinitialize(&self) -> impl core::future::Future<Output = ()> + Send {async {
        
    }}
}

/// # [`ActorContext`]
/// Provides an actor with access to the system and to metadata about itself
pub struct ActorContext<D> {
    /// The underlying system
    pub(crate) system: Fluxion<D>,
    /// The actor's id
    pub(crate) id: u64,
}

impl<D: Delegate> ActorContext<D> {
    /// # [`ActorContext::get_id`]
    /// Returns the id of the actor
    #[must_use]
    pub fn get_id(&self) -> u64 {
        self.id
    }

    /// # [`ActorContext::system`]
    /// Returns the Fluxion instance that this actor is running on
    #[must_use]
    pub fn system(&self) -> &Fluxion<D> {
        &self.system
    }
}

/// # [`Handler`]
pub trait Handler<M: Message>: Actor {
    fn handle_message<D: Delegate>(&self, message: M, context: &ActorContext<D>) -> impl core::future::Future<Output = M::Result> + Send;
}




/// Newtype pattern implementing Slacktor's actor trait
/// for implementorrs of our [`Actor`] trait here.
pub(crate) struct ActorWrapper<T: Actor, D: Delegate>(pub T, pub Arc<ActorContext<D>>);

impl<R: Actor, D: Delegate> slacktor::Actor for ActorWrapper<R, D> {
    fn destroy(&self) -> impl core::future::Future<Output = ()> + Send {
        self.0.deinitialize()
    }
}

impl<R: Handler<M>, M: Message, D: Delegate> slacktor::actor::Handler<M> for ActorWrapper<R, D> {
    #[inline]
    fn handle_message(&self, message: M) -> impl core::future::Future<Output = <M as Message>::Result> + Send {
        self.0.handle_message(message, &self.1)
    }
}

