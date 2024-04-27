use fluxion::{Actor, Fluxion, Handler, Identifier, Message, MessageSender};
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
    async fn handle_message(&self, message: TestMessage) {
        println!("Test message");
    }
}

impl Handler<SerializedMessage> for TestActor {
    async fn handle_message(&self, message: SerializedMessage) {
        println!("Test serialized message");
    }
}

struct SimpleDelegate(Fluxion<()>);


#[tokio::main]
async fn main() {
    let system1 = Fluxion::new("system1", ());
    

    let id = system1.add(TestActor).await.unwrap();

    let actor = system1.get_local::<TestActor>(id).await.unwrap();
    actor.send(TestMessage).await;
}