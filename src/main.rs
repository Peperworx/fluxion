


use async_trait::async_trait;
use fluxion::{actor::{Actor, ActorMessage, context::ActorContext, MessageHandler}, error::ActorError};

struct TestMessage;

impl ActorMessage for TestMessage {
    type Response = ();
}

struct TestActor;

#[async_trait]
impl Actor for TestActor {
    async fn initialize(&mut self, _context: &mut ActorContext) -> Result<(), ActorError> {
        Ok(())
    }

    async fn deinitialize(&mut self, _context: &mut ActorContext) -> Result<(), ActorError> {
        Ok(())
    }
}

#[async_trait]
impl MessageHandler<TestMessage> for TestActor {
    async fn message(&mut self, _context: &mut ActorContext, _message: TestMessage) -> Result<<TestMessage as ActorMessage>::Response, ActorError> {
        println!("Recieved Message");
        Ok(())
    }
}



#[tokio::main]
async fn main() {
}