

use std::marker::PhantomData;

use async_trait::async_trait;
use fluxion::{actor::{Actor, ActorMessage, context::ActorContext, NotifyHandler, FederatedHandler, MessageHandler}, error::{ActorError, ErrorPolicyCollection}, system::System};

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Clone)]
struct GenericMessage<M: Clone + Send + Sync + 'static, R:  Clone + Send + Sync + 'static> {
    pub data: M,
    _phantom_data: PhantomData<R>
}
impl<M: Clone + Send + Sync + 'static, R:  Clone + Send + Sync + 'static> GenericMessage<M, R> {
    pub fn new(data: M) -> Self {
        Self {
            data,
            _phantom_data: PhantomData::default()
        }
    }
}

impl<M:  Clone + Send + Sync + 'static, R: Clone + Send + Sync + 'static> ActorMessage for GenericMessage<M, R> {
    type Response = R;
}

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

#[async_trait]
impl MessageHandler<GenericMessage<u32, ()>> for TestActor {
    async fn message(&mut self, _context: &mut ActorContext, message: GenericMessage<u32, ()>) -> Result<(), ActorError> {
        println!("Recieved u32 {}", message.data);
        Ok(())
    }
}



#[tokio::main]
async fn main() {
    let sys = System::<(), TestMessage>::new("sys1".to_string());
    
    let ar = sys.add_actor::<GenericMessage<u32,()>, TestActor>(TestActor {}, "test".to_string(), ErrorPolicyCollection::default()).await.unwrap();
    ar.send(GenericMessage::<u32,()>::new(10)).await.unwrap();

    sys.shutdown().await;
}

