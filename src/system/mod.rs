//! # System
//! The system forms the core of Fluxion. It is responsible for the storage, creation, and management of actors.

use core::marker::PhantomData;

use alloc::sync::Arc;
use async_rwlock::RwLock;

use hashbrown::HashMap;

use crate::{
    actor::{actor_ref::ActorRef, entry::ActorEntry, id::ActorId, supervisor::ActorSupervisor},
    error::SystemError,
    message::Message,
    util::generic_abstractions::{ActorParams, SystemParams},
    ActorGenerics, ParamActor,
};

#[cfg(notification)]
use crate::util::generic_abstractions::MessageParams;

#[cfg(notification)]
use crate::Channel;

#[derive(Clone)]
pub struct System<'a, S: SystemParams> {
    /// The map which contains every actor.
    /// This is wrapped in an [`Arc`] and [`RwLock`] to allow it to be accessed from many different tasks.
    actors: Arc<RwLock<HashMap<ActorId, Arc<dyn ActorEntry>>>>,
    /// The notification channel
    #[cfg(notification)]
    notifications: Channel<<S::SystemMessages as MessageParams>::Notification>,
    /// The id of this sytem
    id: &'a str,
    _phantom: PhantomData<S>,
}

impl<'a, S: SystemParams> System<'a, S> {
    #[must_use]
    pub fn new(id: &'a str) -> Self {
        // If notifications are enabled, create the notification channel
        #[cfg(notification)]
        let notifications = Channel::unbounded();

        Self {
            actors: Arc::default(),
            #[cfg(notification)]
            notifications,
            id,
            _phantom: PhantomData,
        }
    }

    /// Returns the system's id
    #[must_use]
    pub fn get_id(&self) -> &str {
        self.id
    }

    /// Adds a new actor to the system and begins running the supervisor
    ///
    /// # Errors
    /// Returns an error if the actor already exists in the system.
    pub async fn add<M: Message, A: ParamActor<M, S>>(
        &self,
        id: ActorId,
        actor: A,
    ) -> Result<Arc<ActorRef<ActorGenerics<A, M>, S>>, SystemError> {
        // Initialize the supervisor
        let mut supervisor = ActorSupervisor::<ActorGenerics<_, _>, S>::new(
            actor,
            #[cfg(notification)]
            self.notifications.clone(),
        );

        // Get the supervisor's reference
        let actor_ref = Arc::new(supervisor.get_ref());

        // Lock the hashmap as write.
        // We do this here so that the actor's initialization
        // can't ever lock it first.
        let mut actors = self.actors.write().await;

        // If the actor exists, error
        if actors.contains_key(&id) {
            return Err(SystemError::ActorExists);
        }

        // Start the supervisor
        agnostik::spawn(async move {
            supervisor.run().await;
            // TODO: Cleanup here.
        });

        // Insert the actor
        // We can do this unchecked, because we already checked if it existed.
        actors.insert_unique_unchecked(id, actor_ref.clone());

        // Return the actor reference
        Ok(actor_ref)
    }
}
