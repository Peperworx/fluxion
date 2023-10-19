

use std::{time::Duration, alloc::System};

use fluxion::{Executor, FluxionParams, Actor, Handler, ActorError, actor::supervisor::Supervisor, system::Fluxion};



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
    // Create the system
    let system = Fluxion::<SystemConfig>::new("host");

    // Add an actor to the system
    let ah = system.add(TestActor, "test").await.unwrap();

    // Send a message
    ah.request(()).await.unwrap();

    // Shutdown the system
    system.shutdown().await;
}