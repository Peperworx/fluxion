


use fluxion::{Executor, FluxionParams, Actor, Handler, ActorError, Fluxion, System};



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

    // We can also get the actor's handle as a trait.
    // This only requires the actor's type when the handle is retrieved, allowing
    // better interoperability with other crates. This comes with the downside of
    // requiring that the message type being used is known, and management functions are not available.
    // If management functions are needed, use `get_local` instead, which returns the same as the `add`
    // method.
    drop(ah);
    let ah = system.get::<TestActor, ()>("test".into()).await.unwrap();

    // Works in the same way
    ah.request(()).await.unwrap();

    // Shutdown the system
    system.shutdown().await;
}