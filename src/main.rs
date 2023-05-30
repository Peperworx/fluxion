
use async_trait::async_trait;
use fluxion::{actor::{Actor, ActorMessage, ActorID}, system::{SystemEvent, System}};


#[derive(Clone, Debug)]
enum TestMessage {
    Ping,
    Pong
}

impl ActorMessage for TestMessage {
    type Response = TestMessage;
}

#[derive(Debug, Clone)]
struct TestEvent;

impl SystemEvent for TestEvent {}

struct TestActor {
    
}

#[async_trait]
impl Actor<TestMessage, TestEvent> for TestActor {
    async fn message(&mut self, _context: &mut fluxion::actor::ActorContext<TestEvent>, message: TestMessage) -> <TestMessage as ActorMessage>::Response {
        match message {
            TestMessage::Ping => TestMessage::Pong,
            TestMessage::Pong => TestMessage::Ping
        }
    }

    async fn event(&mut self, _context: &mut fluxion::actor::ActorContext<TestEvent>, event: TestEvent) {
        println!("Recieved event {event:?}");
    }
}

#[tokio::main]
async fn main() {
    let system = System::new(String::from("sys1"));

    let actor = TestActor {};
    let id: ActorID = String::from("test");

    let _ah = system.add_actor(actor, id).await;
}