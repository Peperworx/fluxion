//! # References
//! [`ActorRef`]s, or Actor References, are the primary method through which actors control each other.

use crate::{Actor, ActorWrapper};


/// # [`ActorRef`]
/// This trait provides methods for actors to communicate with and control each other.
pub trait ActorRef {}

#[repr(transparent)]
pub struct LocalRef<A: Actor>(pub(crate) slacktor::ActorHandle<ActorWrapper<A>>);

impl<A: Actor> ActorRef for LocalRef<A> {}