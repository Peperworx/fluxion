//! Fluxion uses an interesting method of passing complicated generics in some places, replacing them with associated types.
//! In your application, you may wish to "upgrade" these associated types to generics. These contained structs do just that.
//! More complicated applications may, however, wish to just implement their own, which is the recommended method.

use core::marker::PhantomData;

use crate::actor::{Actor, Handle};
use crate::message::Message;

#[cfg(serde)]
use crate::message::serializer::MessageSerializer;

use super::generic_abstractions::{ActorParams, MessageParams, SystemParams};

/// # `ActorGenerics`
/// A simple way to convert [`ActorParams`]' associated types to generics.
pub struct ActorGenerics<A: Actor, M: Message>(PhantomData<(A, M)>);

/// # `ParamActor`
/// Used by [`SupervisorParams`] in conjunction with the [`cfg_matrix`] crate to simplify `#[cfg]`s

#[cfg_attr(any(federated, notification), cfg_matrix::cfg_matrix {
    Handle<<S::SystemMessages as MessageParams>::Federated> : federated,
    Handle<<S::SystemMessages as MessageParams>::Notification>: notification,
})]
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
pub struct SystemGenerics<
    #[cfg(any(federated, notification))] M: MessageParams,
    #[cfg(serde)] SD: MessageSerializer,
>(
    #[cfg(any(federated, notification))] PhantomData<M>,
    #[cfg(serde)] PhantomData<SD>,
);

cfg_if::cfg_if! {
    if #[cfg(all(serde, any(federated, notification)))] {
        impl<M: MessageParams, SD: MessageSerializer> SystemParams for SystemGenerics<M, SD> {
            #[cfg(any(federated, notification))]
            type SystemMessages = M;

            #[cfg(serde)]
            type Serializer = SD;
        }
    } else if #[cfg(serde)] {
        impl<SD: MessageSerializer> SystemParams for SystemGenerics<SD> {
            #[cfg(serde)]
            type Serializer = SD;
        }
    } else if #[cfg(any(federated, notification))] {
        impl<M: MessageParams> SystemParams for SystemGenerics<M> {
            #[cfg(any(federated, notification))]
            type SystemMessages = M;
        }
    } else {
        impl SystemParams for SystemGenerics {
        }
        impl SystemParams for () {}
    }
}

#[cfg(serde)]
#[cfg(not(serde))]
impl<M: MessageParams> SystemParams for SystemGenerics<M> {
    type SystemMessages = M;
}

/// A simple way to convert [`MessageParams`]' associated types to generics
/// Federated messages and notifications are not behind feature flags on this one, because `()` can be substituted
/// for them. This also prevents confusion when the order of generics are changed due to feature flags.
/// For instances in which neither federated messages nor notifications are enabled, [`MessageParams`] will be implemented for `()`.
pub struct MessageGenerics<F: Message, N: Message>(PhantomData<F>, PhantomData<N>);

impl<F: Message, N: Message> MessageParams for MessageGenerics<F, N> {
    #[cfg(federated)]
    type Federated = F;

    #[cfg(notification)]
    type Notification = N;
}

#[cfg(not(any(federated, notification)))]
impl MessageParams for () {}
