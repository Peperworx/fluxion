
use std::any::{Any, self};

use async_trait::async_trait;
use fluxion::actor::{Actor, ActorMessage, ActorError, Context, Handler, SystemEvent};

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

#[async_trait]
impl Handler<u32> for TestActor {
    async fn event(&mut self, _context: Context, event: u32) -> Result<(), ActorError> {
        println!("Recieved u32 {event}");
        Ok(())
    }
}

#[async_trait]
impl Handler<i32> for TestActor {
    async fn event(&mut self, _context: Context, event: i32) -> Result<(), ActorError> {
        println!("Recieved u32 {event}");
        Ok(())
    }
}

async fn run_handle<T: Any + Send + Sync, A: Actor + Handler<T>>(actor: &mut A, context: Context, event: T) -> Result<(), ActorError> {
    <A as Handler<T>>::event(actor, context, event).await
}

#[tokio::main]
async fn main() {
    let mut a1 = TestActor {};
    let ctx = Context {};
}