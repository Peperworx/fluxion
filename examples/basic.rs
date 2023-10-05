use fluxion::{types::{actor::Actor, params::SupervisorParams, message::{Message, MessageSender}, Handle, errors::ActorError}, supervisor::Supervisor};

#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
struct TestMessage;

impl Message for TestMessage {
    type Response = ();
}

struct TestActor;

impl Actor for TestActor {
    type Error = ();
}
#[cfg_attr(async_trait, async_trait::async_trait)]
impl Handle<()> for TestActor {
    async fn message(&self, message: &()) -> Result<(), ActorError<()>> {
        println!("()");
        Ok(())
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl Handle<TestMessage> for TestActor {
    async fn message(&self, message: &TestMessage) -> Result<(), ActorError<()>> {
        println!("TestMessage");
        Ok(())
    }
}

struct SupervisorConfig;
impl SupervisorParams for SupervisorConfig {
    type Actor = TestActor;
}

#[tokio::main(flavor = "current_thread")]
async fn main() {

    // Create a supervisor
    let supervisor = Supervisor::<SupervisorConfig>::new(TestActor);

    // Create a handle
    let handle = supervisor.handle();

    // Erase the message type for two different message types
    let send_empty: &dyn MessageSender<()> = &handle;
    let send_test: &dyn MessageSender<TestMessage> = &handle;

    // Run the supervisor
    let sup = tokio::spawn(async move {
        loop {
            supervisor.tick().await.unwrap();
        }
    });

    // This works
    handle.request(()).await.unwrap();
    handle.request(TestMessage).await.unwrap();

    // But these work also
    send_empty.request(()).await.unwrap();
    send_test.request(TestMessage).await.unwrap();

    sup.await.unwrap();
}