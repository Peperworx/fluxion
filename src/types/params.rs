//! # Params
//! This module contains several traits which are used to conveniently pass around generic parameters which would otherwise become unwieldley.
//! This also allows these parameters to be enabled or disabled depending on feature flags.

use super::{actor::Actor, message::Message};




/// # [`SupervisorParams`]
/// This trait contains parameters and configuration data used by actor supervisors.
pub trait SupervisorParams {

    /// This is the type of the supervised actor.
    type Actor: Actor;

    /// The capacity of the MPSC channel receiving messages.
    /// Defaults to 64.
    const MessageCapacity: usize = 64;
}