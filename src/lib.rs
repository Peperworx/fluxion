#! [doc = include_str! ("../README.md")]


#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(async_trait), feature(async_fn_in_trait))]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]



extern crate alloc;


pub mod types;

pub mod actor;

pub mod message;

pub mod system;

#[cfg(error_policy)]
pub mod error_policy;

// Quick and dirty method to remove requirement for a cfg macro before every tracing event
#[cfg(tracing)]
pub(crate) use tracing::event;
#[cfg(not(tracing))]
#[macro_export]
macro_rules! event {
    ($($t:expr),*) => {
        
    };
}

mod fluxion;

pub use fluxion::*;


pub use types::{
    errors::{ActorError, SendError},
    executor::{Executor, JoinHandle},
    params::FluxionParams,
};

pub use actor::{
    Actor, ActorId,
    context::ActorContext,
};

pub use system::System;

pub use message::{
    Message, Handler,
    inverted::{InvertedHandler, InvertedMessage},
    MessageSender,
    event::Event,
};

#[cfg(serde)]
pub use types::serialize::MessageSerializer;




