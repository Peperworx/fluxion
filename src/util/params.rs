//! Fluxion uses an interesting method of passing complicated generics in some places, replacing them with associated types.
//! In your application, you may wish to "upgrade" these associated types to generics. These contained structs do just that.
//! More complicated applications may, however, wish to just implement their own, which is the recommended method.

use core::marker::PhantomData;

use serde::Serializer;

use crate::actor::{Actor, Handle};
use crate::message::Message;

use crate::message::serializer::MessageSerializer;

use super::generic_abstractions::{ActorParams, MessageParams, SystemParams};

/// # `ActorGenerics`
/// A simple way to convert [`ActorParams`]' associated types to generics.
pub struct ActorGenerics<A: Actor, M: Message>(PhantomData<(A, M)>);

/// # `ParamActor`
/// Used by [`SupervisorParams`] in conjunction with the [`cfg_matrix`] crate to simplify `#[cfg]`s
#[cfg_matrix::cfg_matrix {
    Handle<<S::SystemMessages as MessageParams>::Federated> : federated,
    Handle<<S::SystemMessages as MessageParams>::Notification>: notification
}]
pub trait ParamActor<M: Message, S: SystemParams>: Actor + Handle<M> {}

// Implementing [`ParamActor`] for every type that matches its constraints.
// I am woring on an extention to [`cfg_matrix`] to do this automagically, and will
// replace this once it is finished. For now though, This Just Works :tm:, so it may be a while.

cfg_if::cfg_if! {
    if #[cfg(all(federated, notification))] {
        impl<T, M, S> ParamActor<M, S> for T
        where
            T: Actor
                + Handle<M>
                + Handle<<S::SystemMessages as MessageParams>::Federated>
                + Handle<<S::SystemMessages as MessageParams>::Notification>,
            M: Message,
            S: SystemParams,
        {
        }
    } else if #[cfg(federated)] {
        impl<T, M, S> ParamActor<M, S> for T
        where
            T: Actor
                + Handle<M>
                + Handle<<S::SystemMessages as MessageParams>::Federated>,
            M: Message,
            S: SystemParams,
        {
        }
    } else if #[cfg(notification)] {
        impl<T, M, S> ParamActor<M, S> for T
        where
            T: Actor
                + Handle<M>
                + Handle<<S::SystemMessages as MessageParams>::Notification>,
            M: Message,
            S: SystemParams,
        {
        }
    } else {
        impl<T, M, S> ParamActor<M, S> for T
        where
            T: Actor
                + Handle<M>,
            M: Message,
            S: SystemParams,
        {
        }
    }
}

impl<A: ParamActor<M, S>, M: Message, S: SystemParams> ActorParams<S> for ActorGenerics<A, M> {
    type Message = M;

    type Actor = A;
}

/// # [`SystemGenerics`]
/// A simple way to convert [`SystemParams`]' associated types to generics.
pub struct SystemGenerics<M: MessageParams, #[cfg(serde)] SD: MessageSerializer>(
    PhantomData<M>,
    #[cfg(serde)] PhantomData<SD>,
);

#[cfg(serde)]
impl<M: MessageParams, SD: MessageSerializer> SystemParams for SystemGenerics<M, SD> {
    #[cfg(any(federated, notification))]
    type SystemMessages = M;

    #[cfg(serde)]
    type Serializer = SD;
}

#[cfg(not(serde))]
impl<M: MessageParams> SystemParams for SystemGenerics<M> {
    type SystemMessages = M;
}

/// A simple way to convert [`MessageParams`]' associated types to generics
pub struct MessageGenerics<#[cfg(federated)] F: Message, #[cfg(notification)] N: Message>(
    #[cfg(federated)] PhantomData<F>,
    #[cfg(notification)] PhantomData<N>,
);

cfg_if::cfg_if! {
    if #[cfg(all(federated, notification))] {
        impl<F: Message, N: Message> MessageParams for MessageGenerics<F, N> {
            type Federated = F;

            type Notification = N;
        }
    } else if #[cfg(federated)] {
        impl<F: Message> MessageParams for MessageGenerics<F> {
            type Federated = F;
        }
    } else if #[cfg(notification)] {
        impl<N: Message> MessageParams for MessageGenerics<N> {
            type Notification = N;
        }
    } else {
        impl MessageParams for MessageGenerics {
        }
        impl MessageParams for () {

        }
    }
}
