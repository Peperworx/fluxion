//! # Params
//! This module contains several traits which are used to conveniently pass around generic parameters which would otherwise become unwieldley.
//! This also allows these parameters to be enabled or disabled depending on feature flags.

use crate::Executor;

#[cfg(serde)]
use super::serialize::MessageSerializer;

/// # [`FluxionParams`]
/// Every configurable parameter used by Fluxion.
/// This is used to greatly reduce the number of generic parameters passed to different structures.
pub trait FluxionParams: Send + Sync + 'static {

    /// The async executor to use
    type Executor: Executor;

    /// The serializer to use for foreign messages
    #[cfg(serde)]
    type Serializer: MessageSerializer;

    
}