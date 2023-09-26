#! [doc = include_str! ("../../README.md")]



// The following will be in *every* crate related to fluxion.
#![no_std]
#![cfg_attr(not(async_trait), feature(async_fn_in_trait))]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

extern crate alloc;

