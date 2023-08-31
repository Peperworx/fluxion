//! Contains the implementation of foreign messages

use crate::{actor::{Actor, wrapper::ActorWrapper, supervisor::{ActorSupervisor, SupervisorGenerics}}, error::FluxionError};

use super::{Message, Handler};

use alloc::vec::Vec;

#[cfg(async_trait)]
use alloc::boxed::Box;

/// # `ForeignHandler`
/// An implementor of Handler which handles foreign messages via deserialization.
pub struct ForeignHandler;
