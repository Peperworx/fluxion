#![cfg_attr(feature = "nightly", feature(async_fn_in_trait))]

use fluxion::{
    actor::{supervisor::ActorSupervisor, Actor, ActorContext, Handle},
    error::FluxionError,
    message::{serializer::MessageSerializer, Message},
    Channel, MessageParams, SupervisorParams,
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

    type Error = ();
}

#[async_trait::async_trait]
impl Handle<TestMessage> for TestActor {
    async fn message(
        &mut self,
        message: &TestMessage,
        _context: &mut ActorContext,
    ) -> Result<(), FluxionError<Self::Error>> {
        println!("test");
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handle<()> for TestActor {
    async fn message(
        &mut self,
        message: &(),
        _context: &mut ActorContext,
    ) -> Result<(), FluxionError<Self::Error>> {
        println!("()");
        Ok(())
    }
}

struct BincodeSerializer;

impl MessageSerializer for BincodeSerializer {
    fn deserialize<T: for<'a> serde::Deserialize<'a>, E>(
        message: &[u8],
    ) -> Result<T, FluxionError<E>> {
        bincode::deserialize(message).or(Err(FluxionError::DeserializeError))
    }

    fn serialize<T: serde::Serialize, E>(message: T) -> Result<Vec<u8>, FluxionError<E>> {
        bincode::serialize(&message).or(Err(FluxionError::SerializeError))
    }
}

#[tokio::main]
async fn main() {
    let actor = TestActor;

    let notification_channel = Channel::unbounded();

    let mut supervisor = ActorSupervisor::<
        SupervisorParams<TestActor, BincodeSerializer>,
        MessageParams<TestMessage, (), ()>,
    >::new(actor, notification_channel);

    // Get the ref
    let a = supervisor.get_ref();

    // Run the supervisor
    tokio::spawn(async move {
        supervisor.run().await;
    });
    a.request(TestMessage).await;
}
