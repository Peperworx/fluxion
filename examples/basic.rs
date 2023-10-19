

use std::time::Duration;

use fluxion::{Executor, FluxionParams, Actor, Handler, ActorError, actor::supervisor::Supervisor};



/// Define an executor to use
struct TokioExecutor;

impl Executor for TokioExecutor {
    fn spawn<T>(future: T) -> fluxion::types::executor::JoinHandle<T::Output>
    where
        T: std::future::Future + Send + 'static,
        T::Output: Send + 'static {
        let handle = tokio::spawn(future);
        fluxion::types::executor::JoinHandle { handle: Box::pin(async {
            handle.await.unwrap()
        }) }
    }
}

/// Define system configuration
#[derive(Clone)]
struct SystemConfig;
impl FluxionParams for SystemConfig {
    type Executor = TokioExecutor;
}


struct TestActor;

#[async_trait::async_trait]
impl<C: FluxionParams> Actor<C> for TestActor {
    type Error = ();

    async fn deinitialize(&mut self) -> Result<(), ActorError<()>> {
        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("Deinitialized");
        Ok(())
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<C: FluxionParams> Handler<C, ()> for TestActor {
    async fn message(
        &self,
        _message: &()
    ) -> Result<(), ActorError<Self::Error>> {
        println!("()");
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Create supervisor
    let mut sv = Supervisor::<SystemConfig, _>::new(TestActor);

    // Get the actor handle
    let ah = sv.handle();

    // Start the supervisor
    tokio::spawn(async move {
        sv.run().await.unwrap();
    });

    // Send a message
    ah.request(()).await.unwrap();

    // Shutdown the actor
    ah.shutdown().await;

}