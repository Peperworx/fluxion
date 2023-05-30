
use async_trait::async_trait;
use fluxion::{actor::{Actor, ActorMessage, ActorID, ActorContext}, system::{SystemEvent, System}};


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
    /// Run when the actor is started
    async fn initialize(&mut self, context: &mut ActorContext<TestEvent>) {
        let id = context.metadata.id.clone();
        println!("Initialized actor {id}");
    }

    /// Run when an actor is stopped
    async fn deinitialize(&mut self, context: &mut ActorContext<TestEvent>) {
        let id = context.metadata.id.clone();
        println!("Deinitialized actor {id}");
    }

    async fn message(&mut self, context: &mut ActorContext<TestEvent>, message: TestMessage) -> <TestMessage as ActorMessage>::Response {
        let id = context.metadata.id.clone();
        println!("Actor {id} recieved Message {message:?}");
        match message {
            TestMessage::Ping => TestMessage::Pong,
            TestMessage::Pong => TestMessage::Ping
        }
    }

    async fn event(&mut self, context: &mut ActorContext<TestEvent>, event: TestEvent) {
        let id = context.metadata.id.clone();
        println!("Actor {id} recieved event {event:?}");
    }
}

#[tokio::main]
async fn main() {
    let system = System::new(String::from("sys1"));

    let actor = TestActor {};
    let id: ActorID = String::from("test");

    let ah = system.add_actor(actor, id.clone()).await;

    println!("{:?}", ah.request(TestMessage::Ping).await);

    system.remove_actor(&id).await;

    system.send_event(TestEvent {}).await;

    // Queue the system for shutdown.
    // Cleanup will happen when it is dropped
    system.shutdown().await;
}