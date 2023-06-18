use fluxion::{message::{Message, handler::HandleNotification}, system::System, actor::{path::ActorPath, Actor, supervisor::SupervisorErrorPolicy, context::ActorContext}, error::{policy::ErrorPolicy, ActorError}, error_policy};



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
    async fn initialize(&mut self, context: &mut ActorContext) -> Result<(), ActorError> {
        Ok(())
    }

    /// Called upon actor deinitialization, when the supervisor stops.
    /// Note that this will not be called if the initialize function fails.
    /// For handling cases of initialization failure, use [`Actor::cleanup`]
    async fn deinitialize(&mut self, context: &mut ActorContext) -> Result<(), ActorError> {
        Ok(())
    }


    /// Called when the actor supervisor is killed, either as the result of a graceful shutdown
    /// or if initialization fails.
    async fn cleanup(&mut self) -> Result<(), ActorError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl HandleNotification<()> for TestActor {
    async fn notified(&mut self, context: &mut ActorContext, notification: &()) -> Result<(), ActorError> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let system = System::<TestMessage, ()>::new("host");

    let start = tokio::time::Instant::now();
    for i in 0..1000000 {
        system.add_actor::<TestActor, TestFederated>(TestActor {}, &format!("{i}"), SupervisorErrorPolicy::default()).await.unwrap();
    }
    let end = tokio::time::Instant::now() - start;
    println!("Created actors in {end:?}");

    //system.notify(()).await;
    //system.drain_notify().await;
}