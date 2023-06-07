use async_trait::async_trait;
use fluxion::{actor::{Actor, ActorMessage, context::ActorContext, NotifyHandler}, error::{ActorError, ErrorPolicyCollection}, system::System};
use tokio::{sync::RwLock, time};

static ACTOROK: RwLock<Vec<bool>> = RwLock::const_new(Vec::new());

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
        
        let mut aok = ACTOROK.write().await;
        aok.push(true);
        Ok(())
    }
}



#[tokio::main]
async fn main() {
    
    let sys = System::<()>::new("sys1".to_string());

    let l = 1000000;
    let start = time::Instant::now();
    for i in 0..l {
        sys.add_actor(TestActor {}, i.to_string(), ErrorPolicyCollection::default()).await.unwrap();
    }
    let end = time::Instant::now() - start;
    println!("Initializing {l} actors took {end:?}");

    let start = time::Instant::now();
    sys.notify(());
    sys.drain_notify().await;
    
    let end = time::Instant::now() - start;
    println!("Notifying {l} actors took {end:?}");

    let aok = ACTOROK.read().await;
    println!("{}", aok.len());
    assert!(aok.len() == l);
    assert!(aok.iter().all(|v| *v));
    
    }
    