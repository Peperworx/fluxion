

use std::marker::PhantomData;

use fluxion::{types::{actor::{Actor, ActorId}, params::{SupervisorParams, FluxionParams}, message::{Message, MessageSender}, Handle, errors::ActorError, executor::Executor, context::Context}, supervisor::Supervisor, Fluxion, system::System};

#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
struct TestMessage;

impl Message for TestMessage {
    type Response = ();
}

#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
struct TestMessage2;

impl Message for TestMessage2 {
    type Response = ();
}


struct TestActor<CTX: Context>(PhantomData<CTX>);

impl<CTX: Context> Default for TestActor<CTX> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<CTX: Context> Actor for TestActor<CTX> {
    type Error = ();
    type Context = CTX;
}
#[cfg_attr(async_trait, async_trait::async_trait)]
impl<CTX: Context> Handle<()> for TestActor<CTX> {
    async fn message(&self, message: &(), context: &Self::Context) -> Result<(), ActorError<()>> {
        println!("()");
        Ok(())
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<CTX: Context> Handle<TestMessage> for TestActor<CTX> {
    async fn message(&self, message: &TestMessage, context: &Self::Context) -> Result<(), ActorError<()>> {
        println!("TestMessage");

        let ah = context.get_local::<TestActor<CTX::Context>>(context.get_id().get_actor()).await.unwrap();
        tokio::spawn(async move {
            
        
            ah.request(TestMessage2).await.unwrap();
        });
        
        
        Ok(())
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<CTX: Context> Handle<TestMessage2> for TestActor<CTX> {
    async fn message(&self, message: &TestMessage2, context: &Self::Context) -> Result<(), ActorError<()>> {
        println!("TestMessage2");
        Ok(())
    }
}

struct TokioExecutor;

impl Executor for TokioExecutor {
    fn spawn<T>(&self, future: T) -> fluxion::types::executor::JoinHandle<T::Output>
    where
        T: std::future::Future + Send + 'static,
        T::Output: Send + 'static {
        let handle = tokio::spawn(future);
        fluxion::types::executor::JoinHandle { handle: Box::pin(async {
            handle.await.unwrap()
        }) }
    }
}

#[derive(Clone)]
struct SystemConfig;
impl FluxionParams for SystemConfig {
    type Executor = TokioExecutor;
}

#[tokio::main(flavor = "current_thread")]
async fn main() {

    // Create a system
    let system = Fluxion::<SystemConfig>::new("test", TokioExecutor);

    // Add an actor
    let ah = system.add::<TestActor<_>>(TestActor::default(), "test").await.unwrap();

    // Send some messages
    ah.request(()).await.unwrap();
    ah.request(TestMessage).await.unwrap();

    // Drop the handle
    drop(ah);

    // Get a new local handle
    let ah = system.get_local::<TestActor<_>>("test").await.unwrap();

    // And it also works
    ah.request(()).await.unwrap();
    ah.request(TestMessage).await.unwrap();

    drop(ah);

    let id: ActorId = "test".into();
    println!("{}:{}", id.get_system(), id.get_actor());

    // We can also get a handle that only supports a single message type,
    // but does not require the actor's type to be known after this function is called.
    // This also supports foreign actors.
    let ah = system.get::<TestActor<_>, ()>("test".into()).await.unwrap();

    ah.request(()).await.unwrap();
}