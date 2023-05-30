#![feature(async_fn_in_trait)]

use fluxion::{actor::{Actor, ActorMessage, ActorID, ActorSupervisor}, system::{SystemEvent, System}};


#[derive(Debug)]
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

impl Actor<TestMessage, TestEvent> for TestActor {
    async fn message(&mut self, context: &mut fluxion::actor::ActorContext, message: TestMessage) -> <TestMessage as ActorMessage>::Response {
        match message {
            TestMessage::Ping => TestMessage::Pong,
            TestMessage::Pong => TestMessage::Ping
        }
    }

    async fn event(&mut self, context: &mut fluxion::actor::ActorContext, event: TestEvent) {
        println!("Recieved event {:?}", event);
    }
}

#[tokio::main]
async fn main() {
    let system = System::new(String::from("sys1"));

    let actor = TestActor {};
    let id: ActorID = String::from("test");

    let (mut runner, handle) = ActorSupervisor::new(actor, id);

    tokio::spawn(async move {
        runner.run(system.clone()).await;
    });

    loop {
        let res = handle.request(TestMessage::Ping).await;
        println!("{:?}", res);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let res = handle.request(TestMessage::Pong).await;
        println!("{:?}", res);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}