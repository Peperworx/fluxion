use fluxion::{message::{Message, handler::HandleNotification}, system::System, actor::{ Actor, supervisor::SupervisorErrorPolicy, context::ActorContext}, error:: ActorError};




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
    async fn initialize(&mut self, _context: &mut ActorContext) -> Result<(), ActorError> {
        Ok(())
    }

    /// Called upon actor deinitialization, when the supervisor stops.
    /// Note that this will not be called if the initialize function fails.
    /// For handling cases of initialization failure, use [`Actor::cleanup`]
    async fn deinitialize(&mut self, _context: &mut ActorContext) -> Result<(), ActorError> {
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
    async fn notified(&mut self, _context: &mut ActorContext, _notification: &()) -> Result<(), ActorError> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let system = System::<TestMessage, ()>::new("host");

    let start = std::time::Instant::now();
    let policy = SupervisorErrorPolicy::default();
    for i in 0..100000 {
        system.add_actor::<TestActor, TestFederated>(TestActor {}, &format!("{i}"),  policy.clone()).await.unwrap();
    }
    let end = std::time::Instant::now() - start;
    println!("Created actors in {end:?}");

    let start = std::time::Instant::now();
    system.notify(()).await;
    system.drain_notify().await;
    let end = std::time::Instant::now() - start;
    println!("Notified actors in {end:?}");
}