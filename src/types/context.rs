//! Contains structures that allow an actor to access its outside world.

use crate::{system::System, Fluxion, handle::LocalHandle};

use super::{actor::{ActorId, Actor}, params::FluxionParams, message::{MessageSender, Message}, Handle};

use alloc::boxed::Box;


/// # [`Context`]
/// This trait allows an actor's context to be defined generically, and provides functions for interacting with
/// the system and retrieving details about the actor.

pub trait Context: System + Send + Sync + 'static {
    /// Retrieve's the actor's ID
    fn get_id(&self) -> ActorId;
}

/// # [`ActorContext`]
/// Implements [`Context`] and [`System`].
pub struct ActorContext<Params: FluxionParams> {
    /// The underlying [`Fluxion`] instance, to which calls to the [`System`] trait are deffered.
    system: Fluxion<Params>,
    /// The actor's ID
    id: ActorId,
}

impl<Params: FluxionParams> ActorContext<Params> {
    #[must_use]
    pub fn new(system: Fluxion<Params>, id: ActorId) -> Self {
        Self {
            system, id
        }
    }
}


// The desugared implementation is used here so that we do not cause an additional allocation
// every time these functions are caused.
impl<Params: FluxionParams> System for ActorContext<Params> {

    type Context = Self;

    fn add<'life0,'life1,'async_trait,A: Actor<Context = Self>>(&'life0 self, actor:A, id: &'life1 str) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Option<LocalHandle<A> > > + ::core::marker::Send+'async_trait> >where A:'async_trait+Actor,'life0:'async_trait,'life1:'async_trait,Self:'async_trait {
        self.system.add(actor, id)
    }

    fn get_local<'life0,'life1,'async_trait,A: Actor<Context = Self>>(&'life0 self, id: &'life1 str) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Option<LocalHandle<A> > > + ::core::marker::Send+'async_trait> >where A:'async_trait+Actor,'life0:'async_trait,'life1:'async_trait,Self:'async_trait {
        self.system.get_local(id)
    }


    fn get<'life0,'async_trait,A: Actor<Context = Self>,M>(&'life0 self, id:ActorId) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Option<Box<dyn MessageSender<M> > > > + ::core::marker::Send+'async_trait> >where A:'async_trait+Handle<M> ,M:'async_trait+Message,'life0:'async_trait,Self:'async_trait {
        self.system.get::<A, M>(id)
    }

    fn remove<'life0,'life1,'async_trait>(&'life0 self,id: &'life1 str) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = ()> + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait {
        self.system.remove(id)
    }
}

impl<Params: FluxionParams> Context for ActorContext<Params> {
    fn get_id(&self) -> ActorId {
        self.id.clone()
    }
}