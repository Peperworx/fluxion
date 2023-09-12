//! # [`ActorId`]
//! This module contains [`ActorId`], a per-actor human-readable identifier for actors both on the current system and on other systems.

use alloc::sync::Arc;

/// #[`ActorId`]
///
/// A string in the format `[system:]*actor_id`.
/// `"actor_id"` and `":actor_id"` refer to an actor on the local system. The local system's id may also be used:
/// `"local_system_id:actor_id"`, although this often leads to duplicated code.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActorId(Arc<str>);

impl ActorId {
    /// Get the actor refered to by the [`ActorId`].
    #[must_use]
    pub fn get_actor(&self) -> &str {
        self.0.split(':').last().unwrap_or_default()
    }

    /// Gets the number of non-empty segments in the id
    #[must_use]
    pub fn len(&self) -> usize {
        // Just count every non-empty segment.
        self.0.split(':').filter(|v| !v.is_empty()).count()
    }

    /// Returns true if the actor is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the number of systems in the id
    #[must_use]
    pub fn num_systems(&self) -> usize {
        // The number of systems is equal to the number of segments - 1,
        // except if the number of segments is zero.
        let num_segments = self.len();
        if num_segments > 0 {
            num_segments - 1
        } else {
            0
        }
    }

    /// Gets an iterator over the systems in the id
    pub fn get_systems(&self) -> impl Iterator<Item = &str> {
        // This iterator just takes num_systems worth of systems.
        self.0.split(':').take(self.num_systems())
    }
}

impl AsRef<str> for ActorId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<T> From<T> for ActorId
where
    Arc<str>: From<T>,
{
    fn from(value: T) -> Self {
        ActorId(Arc::<str>::from(value))
    }
}
