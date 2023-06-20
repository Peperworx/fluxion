use fluxion::{
    actor::{context::ActorContext, supervisor::SupervisorErrorPolicy, Actor},
    error::ActorError,
    message::{
        handler::{HandleFederated, HandleMessage, HandleNotification},
        Message, Notification,
    },
    system::System,
};

#[derive(Clone, Debug)]
struct TestMessage;

impl Message for TestMessage {
    type Response = ();
}

#[derive(Clone, Debug)]
struct TestFederated;

impl Message for TestFederated {
    type Response = ();
}

struct TestActor;

#[async_trait::async_trait]
impl Actor for TestActor {
    /// Called upon actor initialization, when the supervisor begins to run.
    async fn initialize<F: Message, N: Notification>(
        &mut self,
        _context: &mut ActorContext<F, N>,
    ) -> Result<(), ActorError> {
        println!("initialize");
        Ok(())
    }

    /// Called upon actor deinitialization, when the supervisor stops.
    /// Note that this will not be called if the initialize function fails.
    /// For handling cases of initialization failure, use [`Actor::cleanup`]
    async fn deinitialize<F: Message, N: Notification>(
        &mut self,
        _context: &mut ActorContext<F, N>,
    ) -> Result<(), ActorError> {
        println!("deinitialize");
        Ok(())
    }

    /// Called when the actor supervisor is killed, either as the result of a graceful shutdown
    /// or if initialization fails.
    async fn cleanup(&mut self) -> Result<(), ActorError> {
        println!("cleanup");
        Ok(())
    }
}

#[async_trait::async_trait]
impl HandleNotification<()> for TestActor {
    async fn notified<F: Message>(
        &mut self,
        _context: &mut ActorContext<F, ()>,
        _notification: (),
    ) -> Result<(), ActorError> {
        println!("notification");
        Ok(())
    }
}

#[async_trait::async_trait]
impl HandleMessage<TestMessage> for TestActor {
    async fn message<F: Message, N: Notification>(
        &mut self,
        context: &mut ActorContext<F, N>,
        _message: TestMessage,
    ) -> Result<(), ActorError> {
        println!("message on {:?}", context.get_path());
        Ok(())
    }
}

#[async_trait::async_trait]
impl HandleFederated<TestFederated> for TestActor {
    async fn federated_message<N: Notification>(
        &mut self,
        context: &mut ActorContext<TestFederated, N>,
        _message: TestFederated,
    ) -> Result<(), ActorError> {
        println!("federated message on {:?}", context.get_path());
        let ar = context
            .get_actor::<TestMessage>("other:test")
            .await
            .unwrap();
        ar.send(TestMessage).await.unwrap();
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let host = System::<TestFederated, ()>::new("host");
    let other = System::<TestFederated, ()>::new("other");

    let mut foreign_host = host.get_foreign().await.unwrap();
    let other2 = other.clone();

    tokio::task::spawn(async move {
        while let Some(message) = foreign_host.recv().await {
            other2.relay_foreign(message).await.unwrap();
        }
    });

    let mut foreign_other = other.get_foreign().await.unwrap();
    let host2 = host.clone();

    tokio::task::spawn(async move {
        while let Some(message) = foreign_other.recv().await {
            host2.relay_foreign(message).await.unwrap();
        }
    });

    host.add_actor(TestActor, "test", SupervisorErrorPolicy::default())
        .await
        .unwrap();
    other
        .add_actor(TestActor, "test", SupervisorErrorPolicy::default())
        .await
        .unwrap();

    let ar = other.get_actor::<TestMessage>("host:test").await.unwrap();
    ar.request_federated(TestFederated).await.unwrap();

    other.shutdown().await;
    host.shutdown().await;
}
