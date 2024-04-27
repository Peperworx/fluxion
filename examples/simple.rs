use std::sync::Arc;

use fluxion::{Actor, ActorContext, Delegate, Fluxion, Handler, Identifier, Message, MessageSender};
use serde::{Deserialize, Serialize};




struct TestActor;

impl Actor for TestActor {
    type Error = ();
}

struct TestMessage;

impl Message for TestMessage {
    type Result = ();
}

#[derive(Serialize, Deserialize)]
struct SerializedMessage;

impl Message for SerializedMessage {
    type Result = ();
}

impl Handler<TestMessage> for TestActor {
    async fn handle_message<D: Delegate>(&self, _message: TestMessage, _context: &ActorContext<D>) {
        println!("Test message");
        
    }
}

impl Handler<SerializedMessage> for TestActor {
    async fn handle_message<D: Delegate>(&self, _message: SerializedMessage, context: &ActorContext<D>) {
        let s = context.system().get::<Self, TestMessage>(context.get_id()).await.unwrap();
        println!("Test serialized message on {}", context.get_id());
        s.send(TestMessage).await;
    }
}

struct SimpleDelegate(Fluxion<()>);


#[tokio::main]
async fn main() {
    let system1 = Fluxion::new("system1", ());
    

    let id = system1.add(TestActor).await.unwrap();

    let actor = system1.get_local::<TestActor>(id).await.unwrap();
    actor.send(SerializedMessage).await;
}