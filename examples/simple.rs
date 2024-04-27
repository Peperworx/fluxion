use fluxion::{Actor, Delegate, Fluxion, Handler, Identifier, Message, MessageSender};
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

impl Delegate for SimpleDelegate {
    async fn get_actor<A: Handler<M>, M: Message>(&self, id: fluxion::Identifier<'_>) -> Option<std::sync::Arc<dyn fluxion::MessageSender<M>>>{
        if let Identifier::Foreign(id, host) = id {
            self.0.get::<A, M>(id).await
        } else {
            None
        }
    }
}

#[tokio::main]
async fn main() {
    let system1 = Fluxion::new("system1", ());
    let system2 = Fluxion::new("system2", SimpleDelegate(system1.clone()));

    let id = system1.add(TestActor).await.unwrap();

    let actor_a = system2.get::<TestActor, _>(Identifier::Foreign(id, "system1")).await.unwrap();

    
    actor_a.send(SerializedMessage).await;

    let actor = system1.get_local::<TestActor>(id).await.unwrap();
    actor.send(TestMessage).await;
}