#![cfg_attr(feature="nightly", feature(async_fn_in_trait))]

use fluxion::{message::Message, actor::{Actor, Handle}};



struct TestMessage;

impl Message for TestMessage {
    type Response = ();
}


struct TestMessage2;

impl Message for TestMessage2 {
    type Response = ();
}


struct TestActor;

impl Actor for TestActor {
    type Error = ();
}
#[cfg_attr(async_trait, async_trait::async_trait)]
impl Handle<TestMessage> for TestActor {
    async fn message(&mut self, message: TestMessage) -> Result<(), Self::Error> {
        println!("message 1");
        Ok(())
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl Handle<TestMessage2> for TestActor {
    async fn message(&mut self, message: TestMessage2) -> Result<(), Self::Error> {
        println!("message 2");
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    
}