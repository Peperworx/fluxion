#! [doc = include_str! ("../README.md")]


#![cfg_attr(not(test), no_std)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]


extern crate alloc;

mod fluxion;
pub use fluxion::*;

mod identifiers;
pub use identifiers::*;