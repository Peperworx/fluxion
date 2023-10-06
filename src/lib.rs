#! [doc = include_str! ("../README.md")]


#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(async_trait), feature(async_fn_in_trait))]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use alloc::{collections::BTreeMap, sync::Arc, boxed::Box};
use handle::ActorHandle;

extern crate alloc;


pub mod types;

pub mod supervisor;

pub mod handle;

pub mod system;


type ActorMap = BTreeMap<Arc<str>, Box<dyn ActorHandle>>;
