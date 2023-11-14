


use std::fmt::{Display, Debug};

use fluxion::{Executor, FluxionParams, Actor, Handler, ActorError, Fluxion, System, ActorContext, Message, MessageSerializer, Event, error_policy::ErrorPolicy};
use serde::{Serialize, Deserialize};
use tracing_subscriber::prelude::*;
use color_eyre::eyre::Result;

/// Define an executor to use
struct TokioExecutor;

impl Executor for TokioExecutor {
    fn spawn<T>(future: T) -> fluxion::types::executor::JoinHandle<T::Output>
    where
        T: std::future::Future + Send + 'static,
        T::Output: Send + 'static {
        let handle = tokio::spawn(future);
        fluxion::types::executor::JoinHandle { handle: Box::pin(async {
            handle.await.unwrap()
        }) }
    }
}

/// Define a bincode serializer
struct BincodeSerializer;

impl MessageSerializer for BincodeSerializer {
    fn deserialize<T: for<'a> Deserialize<'a>>(message: Vec<u8>) -> Option<T> {
        bincode::deserialize(&message).ok()
    }

    fn serialize<T: Serialize>(message: T) -> Option<Vec<u8>> {
        bincode::serialize(&message).ok()
    }
}

/// Define system configuration
#[derive(Clone)]
struct SystemConfig;
impl FluxionParams for SystemConfig {
    type Executor = TokioExecutor;

    type Serializer = BincodeSerializer;
}

#[derive(Serialize, Deserialize, Debug)]
struct TestMessage;

impl Message for TestMessage {
    type Response = ();
}

struct TestActor;

#[async_trait::async_trait]
impl<C: FluxionParams> Actor<C> for TestActor {
    type Error = std::io::ErrorKind;

    const ERROR_POLICY: ErrorPolicy<ActorError<Self::Error>> = fluxion::error_policy! {
        fail;
    };
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<C: FluxionParams> Handler<C, ()> for TestActor {
    async fn message(
        &self,
        _context: &ActorContext<C>,
        message: &Event<()>
    ) -> Result<(), ActorError<Self::Error>> {
        println!("{} Received {:?} from {:?}", message.target, message.message, message.source);
        Err(ActorError::CustomError(std::io::ErrorKind::Other))
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<C: FluxionParams> Handler<C, TestMessage> for TestActor {
    async fn message(
        &self,
        context: &ActorContext<C>,
        message: &Event<TestMessage>,
    ) -> Result<(), ActorError<Self::Error>> {
        println!("{} Received {:?} from {:?}", message.target, message.message, message.source);
        // Relay to the () handler
        let ah = context.get::<Self, ()>("foreign:test".into()).await.unwrap();
        ah.request(()).await.unwrap();
        Ok(())
    }
}


#[derive(Debug)]
struct WrapErr<E: Debug + Display>(E);

impl<E: Debug + Display> Display for WrapErr<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<E: Debug + Display> std::error::Error for WrapErr<E> {

}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let stdout_log = tracing_subscriber::fmt::layer()
        .pretty();
    tracing_subscriber::registry()
        .with(stdout_log)
        .init();

    // Create a system
    let system = Fluxion::<SystemConfig>::new("host");

    // Create a system for our foreign example
    let foreign = Fluxion::<SystemConfig>::new("foreign");

    // Create a task which will relay foreign messages from system to foreign
    let s1 = system.clone();
    let f1 = foreign.clone();
    tokio::spawn(async move {
        let outbound = s1.outbound_foreign();

        loop {
            let m = outbound.recv().await;

            // Relay the foreign message to f1. There would normally be some more logic in here,
            // especially when sending a message over a network or between processes.
            f1.relay_foreign(m).await.unwrap();
        }
    });

    let s2 = system.clone();
    let f2 = foreign.clone();
    tokio::spawn(async move {
        let outbound = f2.outbound_foreign();

        loop {
            let m = outbound.recv().await;

            // Relay the foreign message to f1. There would normally be some more logic in here,
            // especially when sending a message over a network or between processes.
            s2.relay_foreign(m).await.unwrap();
        }
    });

    // Add an actor on the foreign system
    foreign.add(TestActor, "test").await.unwrap();
    foreign.foreign_proxy::<TestActor, (), ()>("test", "test").await;

    let ah = system.add(TestActor, "test").await.unwrap();

    ah.request(TestMessage).await.map_err(|v| WrapErr(v.to_string()))?;

    

    // Shutdown both systems
    system.shutdown().await;
    foreign.shutdown().await;

    Ok(())
}