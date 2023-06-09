use async_trait::async_trait;
use fluxion::{actor::{Actor, ActorMessage, context::ActorContext, NotifyHandler, FederatedHandler}, error::{ActorError, ErrorPolicyCollection}, system::System};

#[derive(Debug, Clone)]
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
    async fn notified(&mut self, _context: &mut ActorContext, _notification: ()) -> Result<(), ActorError> {
        Ok(())
    }
}

#[async_trait]
impl FederatedHandler<TestMessage> for TestActor {
    async fn federated_message(&mut self, _context: &mut ActorContext, _message: TestMessage) -> Result<(), ActorError> {
        Ok(())
    }
}


#[tokio::main]
async fn main() {
    benchmark(1000000).await;
}
    

async fn benchmark(l: u32) {
    
    let sys = System::<(), TestMessage>::new("sys1".to_string());


    // Create l actors and time it
    let start = tokio::time::Instant::now();
    for i in 0..l {
        sys.add_actor(TestActor {}, i.to_string(), ErrorPolicyCollection::default()).await.unwrap();
    }
    let end = tokio::time::Instant::now() - start;
    println!("Initializing {l} actors took {end:?}");
    println!("\tMean time per actor: {:?}", end/l);
    
    // Notify l actors and time it
    let start = tokio::time::Instant::now();
    sys.notify(());
    sys.drain_notify().await;
    let end = tokio::time::Instant::now() - start;
    println!("Notifying {l} actors took {end:?}");
    println!("\tMean time per actor: {:?}", end/l);

    // Shutdown l actors and time it
    let start = tokio::time::Instant::now();
    sys.shutdown();
    sys.drain_shutdown().await;
    let end = tokio::time::Instant::now() - start;
    println!("Shutting down {l} actors took {end:?}");
    println!("\tMean time per actor: {:?}", end/l);
}