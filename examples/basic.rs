


use fluxion::{Executor, FluxionParams, Actor, Handler, ActorError, Fluxion, System, ActorContext, Message};
use serde::{Serialize, Deserialize};



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

/// Define system configuration
#[derive(Clone)]
struct SystemConfig;
impl FluxionParams for SystemConfig {
    type Executor = TokioExecutor;
}

#[derive(Serialize, Deserialize)]
struct TestMessage;

impl Message for TestMessage {
    type Response = ();
}

struct TestActor;

#[async_trait::async_trait]
impl<C: FluxionParams> Actor<C> for TestActor {
    type Error = ();
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<C: FluxionParams> Handler<C, ()> for TestActor {
    async fn message(
        &self,
        _context: &ActorContext<C>,
        _message: &()
    ) -> Result<(), ActorError<Self::Error>> {
        println!("()");
        Ok(())
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<C: FluxionParams> Handler<C, TestMessage> for TestActor {
    async fn message(
        &self,
        context: &ActorContext<C>,
        _message: &TestMessage
    ) -> Result<(), ActorError<Self::Error>> {
        println!("TestMessage");
        // Relay to the () handler
        let ah = context.get::<Self, ()>(context.get_id()).await.unwrap();
        ah.request(()).await.unwrap();
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Create the system
    let system = Fluxion::<SystemConfig>::new("host");

    // Add an actor to the system
    let ah = system.add(TestActor, "test").await.unwrap();

    // Send a message
    ah.request(()).await.unwrap();

    // We can also get the actor's handle as a trait.
    // This only requires the actor's type when the handle is retrieved, allowing
    // better interoperability with other crates. This comes with the downside of
    // requiring that the message type being used is known, and management functions are not available.
    // If management functions are needed, use `get_local` instead, which returns the same as the `add`
    // method.
    let sender_ah = system.get::<TestActor, ()>("test".into()).await.unwrap();

    // Works in the same way
    sender_ah.request(()).await.unwrap();

    // We can also send messages from message handlers, but we have to use a channel that supports the message.
    // We could get the actor again, or just reuse the local handle.
    ah.request(TestMessage).await.unwrap();

    // Shutdown the system
    system.shutdown().await;
}