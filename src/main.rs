
use async_trait::async_trait;
use fluxion::actor::{Actor, ActorMessage, ActorError, Context};

struct TestMessage;

impl ActorMessage for TestMessage {
    type Response = ();
}

struct TestActor;

#[async_trait]
impl Actor for TestActor {
    type Message = TestMessage;
    type Notify = ();
    type Dynamic = ();

    async fn initialize(&mut self, _context: Context) -> Result<(), ActorError> {
        Ok(())
    }

    async fn deinitialize(&mut self, _context: Context) -> Result<(), ActorError> {
        Ok(())
    }

    async fn notify(&mut self, _context: Context, _notify: Self::Notify) -> Result<(), ActorError> {
        Ok(())
    }

    async fn message(&mut self, _context: Context, _message: Self::Message) -> Result<<Self::Message as ActorMessage>::Response, ActorError> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {

}