#![cfg_attr(feature = "nightly", feature(async_fn_in_trait))]

use std::time::Duration;

use fluxion::{
    actor::{supervisor::ActorSupervisor, Actor, ActorContext, Handle},
    error::{ActorError, MessageError},
    message::{serializer::MessageSerializer, Message},
    system::System,
    ActorGenerics, Channel, MessageGenerics, SystemGenerics, async_executors::{Executor, JoinHandle},
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
        message: &TestMessage,
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
        message: &(),
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

struct TokioExecutor;

impl Executor for TokioExecutor {
    fn spawn<T>(future: T) -> fluxion::async_executors::JoinHandle<T::Output>
    where
        T: futures::Future + Send + 'static,
        T::Output: Send + 'static {
        
        let handle = tokio::spawn(future);
        JoinHandle { handle: Box::pin(async {
            handle.await.unwrap()
        }) }
    }
}


#[tokio::main]
async fn main() {
    // Create the system
    let system = System::<SystemGenerics<TokioExecutor, MessageGenerics<(), ()>, BincodeSerializer>>::new("host");

    // Add an actor to the system
    let ar = system
        .add::<TestMessage, TestActor>("test".into(), TestActor)
        .await
        .unwrap();
    // The actor is now running. Send a message to the actor.
    ar.request(TestMessage).await.unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;
    // The actor is now running. Send a message to the actor.
    ar.request(TestMessage).await.unwrap();
}
