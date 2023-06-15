//! The implementation of systems and surrounding types

use std::{sync::Arc, mem};

use tokio::sync::{mpsc, Mutex};

use crate::message::{foreign::ForeignMessage, Notification, Message};


/// # System
/// The core part of Fluxion, the [`System`] runs actors and handles communications between other systems.
/// 
/// ## Inter-System Communication
/// Fluxion systems enable communication by having what is called a foreign channel.
/// The foreign channel is an mpsc channel, the Reciever for which can be retrieved once by a single outside source using [`System::get_foreign`].
/// When a Message or Foreign Message is sent to an external actor, or a Notification is sent at all, the foreign
/// channel will be notified.
pub struct System<F, N>
where
    F: Message,
    N: Notification, {

    /// The reciever for foreign messages
    /// This uses a Mutex to provide interior mutability.
    foreign_reciever: Arc<Mutex<Option<mpsc::Receiver<ForeignMessage<F, N>>>>>
}

impl<F, N> System<F, N>
where
    F: Message,
    N: Notification {
    
    /// Returns the foreign channel reciever wrapped in an [`Option<T>`].
    /// [`None`] will be returned if the foreign reciever has already been retrieved.
    pub async fn get_foreign(self) -> Option<mpsc::Receiver<ForeignMessage<F, N>>> {
        
        // Lock the foreign reciever
        let mut foreign_reciever = self.foreign_reciever.lock().await;

        // Return the contents and replace with None
        mem::take(std::ops::DerefMut::deref_mut(&mut foreign_reciever))
    }
}