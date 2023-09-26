//! # Params
//! This module contains several traits which are used to conveniently pass around generic parameters which would otherwise become unwieldley.
//! This also allows these parameters to be enabled or disabled depending on feature flags.

use super::{actor::Actor, message::Message};




/// # [`SupervisorParams`]
/// This trait contains parameters which are used by the supervisor.
pub trait SupervisorParams {

    /// This is the type of the supervised actor.
    type Actor: Actor;

    /// This is the type of the notifications received by this actor
    #[cfg(notification)]
    type Notification: Message;
}