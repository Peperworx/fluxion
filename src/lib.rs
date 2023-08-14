#![no_std]
#![cfg_attr(not(async_trait), feature(async_fn_in_trait))]


extern crate alloc;

pub mod actor;

pub mod message;

pub mod error;