use async_trait::async_trait;
use fluxion::{actor::{Actor, ActorMessage, context::ActorContext, NotifyHandler, FederatedHandler}, error::{ActorError, ErrorPolicyCollection}, system::System};

#[derive(Clone)]
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
        println!("deinitialized actor");
        Ok(())
    }
}

#[async_trait]
impl NotifyHandler<()> for TestActor {
    async fn notified(&mut self, _context: &mut ActorContext, _notification: ()) -> Result<(), ActorError> {
        println!("notified");
        Ok(())
    }
}

#[async_trait]
impl FederatedHandler<TestMessage> for TestActor {
    async fn federated_message(&mut self, _context: &mut ActorContext, _message: TestMessage) -> Result<(), ActorError> {
        println!("Recieved federated message");
        Ok(())
    }
}


#[tokio::main]
async fn main() {
    
    let sys = System::<(), TestMessage>::new("sys1".to_string());

    let ar = sys.add_actor(TestActor {}, "test actor".to_string(), ErrorPolicyCollection::default()).await.unwrap();
    
    sys.notify(());
    
}
    