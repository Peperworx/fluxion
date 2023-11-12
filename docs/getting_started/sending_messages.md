---
title: Sending Messages
---

Lets take a look at our main function again:

```rust
#[tokio::main]
async fn main() {
    // Create a Fluxion instance using our configuration, and give it the name `host`
    let system = Fluxion::<FluxionConfig>::new("host");
}
```

Here we have a value `system`, which contains the currently running Fluxion instance.
For our actor to start receiving messages, we first need to start it. This requires using the `System` trait.

```rust
use fluxion::System;

// Add MyActor to the system
let ah = system.add(MyActor, "my_actor").await.unwrap();
```

This spawns a `MyActor`, with the id "my_actor" (well, "host:my_actor"), and returns a "handle", (here `ah` stands for "actor handle") which allows messages to be sent to a specific actor. There are two different types of handles:
- `LocalHandle`s, which have an actor's type associated with them.
- Dynamic handles, which are boxed traits that have a specific message type associated with them

`LocalHandle`s allow using many different message types from the same handle, however they come with the downside of both the actor's type to be known, and that the actor is local. (On the same system).

Dynamic handles on the other hand, support foreign actors, and only require that the message type that is being used to communicate with the actor is known. These only support using a single message type with the actor.

The handle returned by `system.add`, is a `LocalHandle`. Another local handle to an actor can be returned via `system.get_local`, and a dynamic handle to a local or foreign actor can be returned via `system.get`, both working only after an actor has been created.

Now that we have a handle to the actor, we can send a message to it:

```rust
// Send MyMessage to MyActor
ah.request(MyMessage).await.unwrap();
```

The `request` function simply takes a message, sends it to an actor, and waits for a response, returning it when received.
Now we want to stop our actor from running, and wait for it to finish processing all remaining messages:

```rust
ah.shutdown().await;
```

We can run this on every actor automatically by calling it on the system, which is a better practice:

```rust
// Shutdown all actors
system.shutdown().await;
```

All together, our code should look like

```rust
mod config;

use config::FluxionConfig;

use fluxion::{Fluxion, Actor, FluxionParams, Handler, ActorContext, Event, ActorError, Message, System};


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

    // Add MyActor to the system
    let ah = system.add(MyActor, "my_actor").await.unwrap();

    // Send MyMessage to MyActor
    ah.request(MyMessage).await.unwrap();

    // Shutdown all actors
    system.shutdown().await;

}

```