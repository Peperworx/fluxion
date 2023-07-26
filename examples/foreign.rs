use fluxion::{
    ActorContext, SupervisorErrorPolicy, Actor,
    ActorError,
    HandleFederated, HandleMessage, HandleNotification,
    Message, Notification,
    system,
};
use serde::{Serialize, Deserialize};
use tracing::Level;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TestMessage;

impl Message for TestMessage {
    type Response = ();
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TestFederated;

impl Message for TestFederated {
    type Response = ();
}

#[derive(Debug)]
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
        println!("message on {:?}", context.get_id());
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
        println!("federated message on {:?}", context.get_id());
        let ar = context
            .get_actor::<TestMessage>("other:test")
            .await
            .unwrap();
        ar.request(TestMessage).await.unwrap();
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let collector = tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::TRACE)
        // build but do not install the subscriber.
        .finish();

    tracing::subscriber::set_global_default(collector).unwrap();

    let host = system::new::<TestFederated, ()>("host");
    let other = system::new::<TestFederated, ()>("other");

    let mut foreign_host = host.get_foreign().await.unwrap();
    let other2 = other.clone();

    let relay_host = tokio::task::spawn(async move {
        while let Some(message) = foreign_host.recv().await {
            other2.relay_foreign(message).await.unwrap();
        }
    });

    
    let mut foreign_other = other.get_foreign().await.unwrap();
    let host2 = host.clone();

    let relay_other = tokio::task::spawn(async move {
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


    // Note that at this point, the task relaying foreign messages has a chance to relay the foreign message *after* the system has been shutdown.
    // This may cause dropped messages.
    // This can be negated by ensuring that the foreign recievers have drained before shutting down, or by simply using `request` instead of `send`
    // for any critical messages.
    // Additionally, a notification can be sent here and drained, which will await until all actors process the notification,
    // allowing actors to finish handling the current message.
    // This example just uses the method of using `request` instead of `send`, and aborting the relaying task, which prevents runtime panics due to the unwraps.
    // All messages sent using `request` will block until they return, and all messages sent using `send` will not panic.
    // Note however that a `request` made in response to a `send` may not block, and in this case the notification method is better.
    // Fluxion does not mandate a specific behavior to handle this, as different implementations may prefer different behaviors.
    relay_host.abort();
    relay_other.abort();

    other.shutdown().await;
    host.shutdown().await;
}
