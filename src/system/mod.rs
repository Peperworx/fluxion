//! # System
//! The system forms the core of Fluxion. It is responsible for the storage, creation, and management of actors.

use core::marker::PhantomData;

use alloc::{string::String, sync::Arc};
use async_rwlock::RwLock;

use hashbrown::HashMap;

use crate::{
    actor::{supervisor::ActorSupervisor, Actor},
    message::Message,
    util::generic_abstractions::{MessageParams, SystemParams},
    ActorGenerics, Channel, ParamActor,
};

#[derive(Clone)]
pub struct System<'a, S: SystemParams> {
    /// The map which contains every actor.
    /// This is wrapped in an [`Arc`] and [`RwLock`] to allow it to be accessed from many different tasks.
    actors: Arc<RwLock<HashMap<String, ()>>>,
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
            _phantom: PhantomData::default(),
        }
    }

    /// Returns the system's id
    #[must_use]
    pub fn get_id(&self) -> &str {
        self.id
    }

    /// Adds a new actor to the system and begins running the supervisor
    pub fn add<A: ParamActor<M, S>, M: Message>(&self, id: &str, actor: A) {
        // Initialize the supervisor
        let supervisor = ActorSupervisor::<ActorGenerics<_, _>, S>::new(
            actor,
            #[cfg(notification)]
            self.notifications.clone(),
        );

        // Get the supervisor's reference
        let actor_ref = supervisor.get_ref();

        todo!()
    }
}
