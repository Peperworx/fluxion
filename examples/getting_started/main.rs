mod config;

use config::FluxionConfig;

use fluxion::{Fluxion, Actor, FluxionParams, Handler, ActorContext, Event, ActorError, Message, System};


/// Our actor, which can contain any data we want. For now we will leave it empty.
struct MyActor;

#[async_trait::async_trait]
impl<C: FluxionParams> Actor<C> for MyActor {
    type Error = ();

    const ErrorPolicy: ErrorPolicy<ActorError<Self::Error>> = ErrorPolicy::default_policy();
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<C: FluxionParams> Handler<C, MyMessage> for  MyActor {

    async fn message(
        &self,
        _context: &ActorContext<C>,
        _message: &Event<MyMessage>
    ) -> Result<(), ActorError<Self::Error>> {

        // Handle the message here...
        println!("Received message!");
        Ok(())
    }
}

/// The message that our actor will handle
#[derive(Debug)]
struct MyMessage;

/// Implement `Message`, which just tells Fluxion what the response type of the message is.
impl Message for MyMessage {
    type Response = ();
}

#[tokio::main]
async fn main() {
    // Create a Fluxion instance using our configuration, and give it the name `host`
    let system = Fluxion::<FluxionConfig>::new("host");

    // Add MyActor to the system
    let ah = system.add(MyActor, "my_actor").await.unwrap();

    // Send MyMessage to MyActor
    ah.request(MyMessage).await.unwrap();

    // Shutdown all actors
    system.shutdown().await;

}