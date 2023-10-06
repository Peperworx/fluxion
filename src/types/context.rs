//! Contains structures that allow an actor to access its outside world.

use crate::{handle::ActorHandle, system::Fluxion, ActorMap};

use super::{executor::Executor, params::FluxionParams};

use alloc::{boxed::Box, sync::Arc};
use maitake_sync::RwLock;
