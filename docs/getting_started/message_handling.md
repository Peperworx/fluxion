---
title: Handling Messages
---

To handle a message, Fluxion actors must implement the `Handler` trait.

To do this, we first import the `Handler` trait, as well as a few extra structs, and then simply implement it for the actor:

```rust
use fluxion::{Handler, ActorContext, Event, ActorError};

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
```

Here we see a single function called `message`, which takes two parameters: `context` and `message`. `context` allows the actor to interact with its environment, and can be used just like the `Fluxion` instance, as we can see in just a minute. The `message` contains both the contents of the message itself, the id of the actor that sent the message, and the id of the message's intended recipient. The message's sender will block until this function completes, and the returned value inside of the result will be sent as the message's response.

Now our entire code should look like this:

```rust
mod config;

use config::FluxionConfig;

use fluxion::{Fluxion, Actor, FluxionParams, Handler, ActorContext, Event, ActorError, Message};


/// Our actor, which can contain any data we want. For now we will leave it empty.
struct MyActor;

#[async_trait::async_trait]
impl<C: FluxionParams> Actor<C> for MyActor {
    type Error = ();
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
}

```