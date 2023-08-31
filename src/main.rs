#![cfg_attr(feature = "nightly", feature(async_fn_in_trait))]

use std::marker::PhantomData;

use fluxion::{
    actor::{
        supervisor::{ActorSupervisor, SupervisorGenerics},
        Actor, ActorContext, Handle,
    },
    error::FluxionError,
    message::{Message, MessageSerializer},
};
use serde::{Deserialize, Serialize};

struct TestNotification;

impl Message for TestNotification {
    type Response = ();
}

#[derive(Serialize, Deserialize)]
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
    async fn message(
        &mut self,
        _message: &TestMessage,
        _context: &mut ActorContext,
    ) -> Result<(), FluxionError<()>> {
        println!("1");
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handle<TestNotification> for TestActor {
    async fn message(
        &mut self,
        _message: &TestNotification,
        _context: &mut ActorContext,
    ) -> Result<(), FluxionError<()>> {
        println!("notification");
        Ok(())
    }
}

struct BincodeSerializer;

impl MessageSerializer for BincodeSerializer {
    fn deserialize<T: for<'a> serde::Deserialize<'a>, E>(
        message: Vec<u8>,
    ) -> Result<T, FluxionError<E>> {
        bincode::deserialize(&message).or(Err(FluxionError::DeserializeError))
    }

    fn serialize<T: serde::Serialize, E>(message: T) -> Result<Vec<u8>, FluxionError<E>> {
        bincode::serialize(&message).or(Err(FluxionError::SerializeError))
    }
}

struct FluxionParams<A, M, N>(PhantomData<A>, PhantomData<M>, PhantomData<N>);

impl<M: Message + Serialize + for<'a> Deserialize<'a>, A: Actor + Handle<M>, N: Message>
    SupervisorGenerics for FluxionParams<A, M, N>
{
    type Actor = A;

    type Serializer = BincodeSerializer;

    type Foreign = M;

    type Notification = N;
}

#[tokio::main]
async fn main() {
    let actor = TestActor;

    let (mut supervisor, actorref) =
        ActorSupervisor::<FluxionParams<TestActor, TestMessage, TestNotification>>::new(actor);

    let jh = tokio::spawn(async move {
        supervisor.run().await.unwrap();
    });

    actorref.request(TestMessage).await.unwrap();
    actorref.request(TestNotification).await.unwrap();

    jh.await.unwrap();
}
