//! The system ([`Fluxion`]) contains every locally running actor supervisor, and provides methods to manage them.

use core::marker::PhantomData;

use alloc::{collections::BTreeMap, sync::Arc, boxed::Box, vec::Vec};
use maitake_sync::RwLock;

use crate::{actor::handle::ActorHandle, FluxionParams};


/// The type alias for the map of actors stored in the system.
type ActorMap = BTreeMap<Arc<str>, Box<dyn ActorHandle>>;


/// # [`Fluxion`]
/// The core management functionality of fluxion.
/// Handles the creation, shutdown, and management of actors.
pub struct Fluxion<C: FluxionParams> {
    /// The map of actors
    actors: Arc<RwLock<ActorMap>>,
    /// The system's ID
    id: Arc<str>,
    /// Phantom data associating the generics with this struct
    _phantom: PhantomData<C>,
}

impl<C: FluxionParams> Fluxion<C> {
    /// Creates a new [`Fluxion`] instance
    #[must_use]
    pub fn new(id: &str) -> Self {
        Fluxion {
            actors: Arc::new(RwLock::new(BTreeMap::default())),
            id: id.into(),
            _phantom: PhantomData,
        }
    }

    /// Gets the system's id
    /// 
    /// # Returns
    /// Returns the local system's id.
    #[must_use]
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Shutdown all local actors
    /// 
    /// # Returns
    /// Returns the number of actors shutdown
    pub async fn shutdown(&self) -> usize {
        // Lock actors as write
        let mut actors = self.actors.write().await;

        // Create a list of shutdown_receivers
        let mut shutdown_receivers = Vec::new();

        // Begin the shutdown process of each actor
        while let Some((_, actor)) = actors.pop_first() {
            // Shutdown the actor and push to shutdown_receivers
            if let Some(receiver) = actor.begin_shutdown().await {
                shutdown_receivers.push(receiver);
            }
        }

        // Count the number of shutdown actors
        let shutdown_actors = shutdown_receivers.len();

        // Wait for every actor to shutdown
        for rcv in shutdown_receivers {
            // We don't care about any errors.
            let _ = rcv.await;
        }

        // Return the number of shutdown actors.
        shutdown_actors
    }
}