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


pub use types::{
    errors::{ActorError, SendError},
    executor::{Executor, JoinHandle},
    params::FluxionParams,
};

pub use actor::{
    Actor, ActorId,
    context::{
        Context, ActorContext
    }
};

pub use message::{
    Message, Handler,
    inverted::{InvertedHandler, InvertedMessage},
    MessageSender
};


//type ActorMap = BTreeMap<Arc<str>, Box<dyn ActorHandle>>;

