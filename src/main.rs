use fluxion::{message::Message, actor::{Actor, Handle, Handled}};



struct TestMessage;

impl Message for TestMessage {
    type Response = ();
}


struct TestMessage2;

impl Message for TestMessage2 {
    type Response = ();
}


struct TestActor;

impl Actor for TestActor {
    type Error = ();
}
#[async_trait::async_trait]
impl Handle<TestMessage> for TestActor {
    async fn message(&mut self, message: TestMessage) -> Result<(), Self::Error> {
        println!("message 1");
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handle<TestMessage2> for TestActor {
    async fn message(&mut self, message: TestMessage2) -> Result<(), Self::Error> {
        println!("message 2");
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    // Create an actor
    let mut actor = TestActor;

    // Directly send a message to the actor
    actor.message(TestMessage).await.unwrap();

    // Directly send a different message to the actor
    actor.message(TestMessage2).await.unwrap();

    // Indirectly send a message to the actor
    TestMessage.handle(&mut actor).await.unwrap();

    // Indirectly send a different message to the actor
    TestMessage2.handle(&mut actor).await.unwrap();
}