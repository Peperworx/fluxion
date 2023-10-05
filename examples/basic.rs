

use fluxion::{types::{actor::{Actor, ActorId, ActorContext}, params::{SupervisorParams, FluxionParams}, message::{Message, MessageSender}, Handle, errors::ActorError, executor::Executor}, supervisor::Supervisor, system::System};

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

struct TestActor;

impl Actor for TestActor {
    type Error = ();

    type Params = SystemConfig;
}
#[cfg_attr(async_trait, async_trait::async_trait)]
impl Handle<()> for TestActor {
    async fn message(&self, message: &(), context: &ActorContext<Self::Params>) -> Result<(), ActorError<()>> {
        println!("()");
        Ok(())
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl Handle<TestMessage> for TestActor {
    async fn message(&self, message: &TestMessage, context: &ActorContext<Self::Params>) -> Result<(), ActorError<()>> {
        println!("TestMessage");
        let context: ActorContext<_> = context.clone();
        tokio::spawn(async move {
            let ah = context.get::<Self, TestMessage2>("test".into()).await.unwrap();
            ah.request(TestMessage2).await.unwrap();
        });
        
        Ok(())
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl Handle<TestMessage2> for TestActor {
    async fn message(&self, message: &TestMessage2, context: &ActorContext<Self::Params>) -> Result<(), ActorError<()>> {
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
    let system = System::<SystemConfig>::new("test", TokioExecutor);

    // Add an actor
    let ah = system.add(TestActor, "test").await.unwrap();

    // Send some messages
    ah.request(()).await.unwrap();
    ah.request(TestMessage).await.unwrap();

    // Drop the handle
    drop(ah);

    // Get a new local handle
    let ah = system.get_local::<TestActor>("test").await.unwrap();

    // And it also works
    ah.request(()).await.unwrap();
    ah.request(TestMessage).await.unwrap();

    drop(ah);

    let id: ActorId = "test".into();
    println!("{}:{}", id.get_system(), id.get_actor());

    // We can also get a handle that only supports a single message type,
    // but does not require the actor's type to be known after this function is called.
    // This also supports foreign actors.
    let ah = system.get::<TestActor, ()>("test".into()).await.unwrap();

    ah.request(()).await.unwrap();
}