#![cfg_attr(feature="nightly", feature(async_fn_in_trait))]

use fluxion::{message::Message, actor::{Actor, Handle, ActorContext, wrapper::ActorWrapper}, error::FluxionError};

struct TestMessage2;

impl Message for TestMessage2 {
    type Response = ();
}

struct TestMessage;

impl Message for TestMessage {
    type Response = ();
}

struct TestActor;

#[async_trait::async_trait]
impl Actor for TestActor {
    type Error = ();
}

#[async_trait::async_trait]
impl Handle<TestMessage> for TestActor {
    async fn message(&mut self, _message: TestMessage, _context: &mut ActorContext) -> Result<(), FluxionError<()>> {
        println!("1");
        Ok(())
    }
}


#[async_trait::async_trait]
impl Handle<TestMessage2> for TestActor {
    async fn message(&mut self, _message: TestMessage2, _context: &mut ActorContext) -> Result<(), FluxionError<()>> {
        println!("2");
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let actor = TestActor;

    let mut actorw = ActorWrapper::new(actor);

    actorw.dispatch(TestMessage).await.unwrap();
}