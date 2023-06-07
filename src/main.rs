


use async_trait::async_trait;
use fluxion::{actor::{Actor, ActorMessage, context::ActorContext, MessageHandler, NotifyHandler}, error::{ActorError, ErrorPolicyCollection}, system::System};

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
impl NotifyHandler<()> for TestActor {
    async fn notified(&mut self, context: &mut ActorContext, _notification: ()) -> Result<(), ActorError> {
        let id = context.get_id();
        println!("Actor {id} recieved a notification.");
        Ok(())
    }
}



#[tokio::main]
async fn main() {

    let sys = System::<()>::new("sys1".to_string());



    let _ar = sys.add_actor(TestActor {}, "test1".to_string(), ErrorPolicyCollection::default()).await.unwrap();
    let _ar = sys.add_actor(TestActor {}, "test2".to_string(), ErrorPolicyCollection::default()).await.unwrap();
    let _ar = sys.add_actor(TestActor {}, "test3".to_string(), ErrorPolicyCollection::default()).await.unwrap();
    println!("{}", sys.notify(()));
}