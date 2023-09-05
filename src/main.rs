#![cfg_attr(feature = "nightly", feature(async_fn_in_trait))]

use fluxion::{
    actor::{Actor, Handle},
    message::Message,
};

struct TestActor;

impl Actor for TestActor {
    type Error = ();
}

struct TestMessage;

impl Message for TestMessage {
    type Response = ();

    type Error = ();
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl Handle<TestMessage> for Actor {
    async fn message(
        &mut self,
        message: &M,
        _context: &mut ActorContext,
    ) -> Result<M::Response, FluxionError<Self::Error>> {
        todo!()
    }
}

#[tokio::main]
async fn main() {}
