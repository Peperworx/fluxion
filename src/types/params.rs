//! # Params
//! This module contains several traits which are used to conveniently pass around generic parameters which would otherwise become unwieldley.
//! This also allows these parameters to be enabled or disabled depending on feature flags.

use core::marker::PhantomData;

use super::{actor::Actor, executor::Executor};




/// # [`SupervisorParams`]
/// This trait contains parameters and configuration data used by actor supervisors.
pub trait SupervisorParams {

    /// This is the type of the supervised actor.
    type Actor: Actor;
}

/// # [`SupervisorGenerics`]
/// A struct that converts the associated types on [`SupervisorParams`] to generics
pub struct SupervisorGenerics<Actor: super::actor::Actor>(PhantomData<Actor>);

impl<Actor: super::actor::Actor> SupervisorParams for SupervisorGenerics<Actor> {
    type Actor = Actor;
}

/// # [`SystemParams`]
/// This trait contains parameters and configuration data used by the system
pub trait SystemParams: Send + Sync + 'static {

    /// The async executor to use
    type Executor: Executor;
}