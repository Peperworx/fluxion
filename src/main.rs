#![cfg_attr(feature = "nightly", feature(async_fn_in_trait))]

use fluxion::{
    actor::{supervisor::ActorSupervisor, Actor, ActorContext, Handle},
    error::{ActorError, MessageError},
    message::{serializer::MessageSerializer, Message},
    ActorGenerics, Channel, SystemGenerics,
};

use serde::{Deserialize, Serialize};

struct TestActor;

impl Actor for TestActor {
    type Error = ();
}

#[derive(Serialize, Deserialize)]
struct TestMessage;

impl Message for TestMessage {
    type Response = ();
}

#[async_trait::async_trait]
impl Handle<TestMessage> for TestActor {
    async fn message(
        &mut self,
        _message: &TestMessage,
        _context: &mut ActorContext,
    ) -> Result<(), ActorError<Self::Error>> {
        println!("test");
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handle<()> for TestActor {
    async fn message(
        &mut self,
        _message: &(),
        _context: &mut ActorContext,
    ) -> Result<(), ActorError<Self::Error>> {
        println!("()");
        Ok(())
    }
}

struct BincodeSerializer;

impl MessageSerializer for BincodeSerializer {
    fn deserialize<T: for<'a> serde::Deserialize<'a>>(message: &[u8]) -> Result<T, MessageError> {
        bincode::deserialize(message).or(Err(MessageError::DeserializeError))
    }

    fn serialize<T: serde::Serialize>(message: T) -> Result<Vec<u8>, MessageError> {
        bincode::serialize(&message).or(Err(MessageError::SerializeError))
    }
}

#[tokio::main]
async fn main() {
    let actor = TestActor;

    let notification_channel = Channel::unbounded();

    let mut supervisor = ActorSupervisor::<
        ActorGenerics<TestActor, TestMessage>,
        SystemGenerics<(), BincodeSerializer>,
    >::new(actor, notification_channel.clone());

    // Get the ref
    let a = supervisor.get_ref();

    // Run the supervisor
    tokio::spawn(async move {
        supervisor.run().await;
    });
    a.request(TestMessage).await.unwrap();

    notification_channel.0.send_async(()).await.unwrap();
}
