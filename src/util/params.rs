//! Fluxion uses an interesting method of passing complicated generics in some places, replacing them with associated types.
//! In your application, you may wish to "upgrade" these associated types to generics. These contained structs do just that.
//! More complicated applications may, however, wish to just implement their own, which is the recommended method.

use core::marker::PhantomData;

use crate::actor::{supervisor::SupervisorGenerics, Actor, Handle};
use crate::message::{Message, MessageGenerics};

use crate::message::serializer::MessageSerializer;

/// # `SupervisorParams`
/// A simply way to convert [`SupervisorGenerics`]' associated types to generics.
pub struct SupervisorParams<A, S>(PhantomData<(A, S)>);

/// # `ParamActor`
/// Used by [`SupervisorParams`] in conjunction with the [`cfg_matrix`] crate to simplify `#[cfg]`s
#[cfg_matrix::cfg_matrix {
    Handle<M::Federated> : federated,
    Handle<M::Notification>: notification
}]
pub trait ParamActor<M: MessageGenerics>: Actor + Handle<M::Message> {}

// Implementing [`ParamActor`] for every type that matches its constraints.
// I am woring on an extention to [`cfg_matrix`] to do this automagically, and will
// replace this once it is finished. For now though, This Just Works :tm:, so it may be a while.

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

impl<A: ParamActor<M>, #[cfg(serde)] S: MessageSerializer, M: MessageGenerics> SupervisorGenerics<M>
    for SupervisorParams<A, S>
{
    type Actor = A;
    #[cfg(serde)]
    type Serializer = S;
}

/// # `MessageParams`
/// A simply way to convert [`MessageGenerics`]' associated types to generics.
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
