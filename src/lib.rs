#![no_std]
#![cfg_attr(not(async_trait), feature(async_fn_in_trait))]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use core::marker::PhantomData;

use actor::{supervisor::SupervisorGenerics, Actor, Handle};
use message::{Message, MessageGenerics};

extern crate alloc;

pub mod actor;

pub mod error;

pub mod message;

/// A utility used to store a two-way channel concisely
pub struct Channel<T>(pub flume::Sender<T>, pub flume::Receiver<T>);

impl<T> Channel<T> {
    /// Creates a new unbounded channel
    #[must_use]
    pub fn unbounded() -> Self {
        let c = flume::unbounded();
        Self(c.0, c.1)
    }
}

/// # `SupervisorParams`
/// A simply way to convert [`SupervisorGenerics`]' associated types to generics.
pub struct SupervisorParams<A, S>(PhantomData<(A, S)>);

/// # `ParamActor`
/// Used by [`SupervisorParams`] in conjunction with the [`cfg_matrix`] crate to simplify cfgs
#[cfg_matrix::cfg_matrix {
    actor::Handle<M::Federated> : federated,
    actor::Handle<M::Notification>: notification
}]
pub trait ParamActor<M: MessageGenerics>: Actor + Handle<M::Message> {}

// I am woring on an extention to [`cfg_matrix`] to do this automagically

cfg_if::cfg_if! {
    if #[cfg(all(federated, notification))] {
        impl<T: Actor + Handle<M::Message> + Handle<M::Federated> + Handle<M::Notification>, M>
            ParamActor<M> for T
        where
            M: MessageGenerics,
        {
        }
    } else if #[cfg(federated)] {
        impl<T: Actor + Handle<M::Message> + Handle<M::Federated>, M>
            ParamActor<M> for T
        where
            M: MessageGenerics,
        {
        }
    } else if #[cfg(notification)] {
        impl<T: Actor + Handle<M::Message> + Handle<M::Notification>, M>
            ParamActor<M> for T
        where
            M: MessageGenerics,
        {
        }
    } else {
        impl<T: Actor + Handle<M::Message>, M>
            ParamActor<M> for T
        where
            M: MessageGenerics,
        {
        }
    }
}

impl<
        A: ParamActor<M>,
        #[cfg(serde)] S: message::serializer::MessageSerializer,
        M: MessageGenerics,
    > SupervisorGenerics<M> for SupervisorParams<A, S>
{
    type Actor = A;
    #[cfg(serde)]
    type Serializer = S;
}

pub struct MessageParams<M, N, F>(PhantomData<(M, N, F)>);

impl<M: Message, #[cfg(notification)] N: Message, #[cfg(federated)] F: Message> MessageGenerics
    for MessageParams<M, N, F>
{
    type Message = M;

    #[cfg(notification)]
    type Notification = N;

    #[cfg(federated)]
    type Federated = F;
}

impl Message for () {
    type Response = ();

    type Error = ();
}
