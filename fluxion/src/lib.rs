#! [doc = include_str! ("../../README.md")]


#![cfg_attr(not(test), no_std)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]


extern crate alloc;

pub use fluxion_macro::message;
pub use const_format::concatcp;

mod fluxion;
pub use fluxion::*;

mod identifiers;
pub use identifiers::*;

mod actor;
pub use actor::*;

mod message;
pub use message::*;

mod references;
pub use references::*;

mod foreign;
pub use foreign::*;


pub use slacktor::Message;