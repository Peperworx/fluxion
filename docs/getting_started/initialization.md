---
title: Initializing Fluxion
---

## Initializing the Instance

Now that we have our `FluxionConfig` struct defined in the `config` module, we can initialize a Fluxion instance in main.rs:

```rust
mod config;

use config::FluxionConfig;

use fluxion::Fluxion;

#[tokio::main]
async fn main() {
    // Create a Fluxion instance using our configuration, and give it the name `host`
    let system = Fluxion::<FluxionConfig>::new("host");
}

```

That was pretty easy! We just used the `tokio::main` proc macro to make the main function async (we will use it later), and created a new fluxion instance.

## Defining Actors

Now, actually defining messages and actors is a little more complex. Actors require that `FluxionConfig` generic, however to make it easier for other libraries to use an actor, we can simply make that a generic:

```rust
mod config;

use config::FluxionConfig;

use fluxion::{Fluxion, Actor, FluxionParams};

/// Our actor, which can contain any data we want. For now we will leave it empty.
struct MyActor;

#[async_trait::async_trait]
impl<C: FluxionParams> Actor<C> for MyActor {
    type Error = ();
}

#[tokio::main]
async fn main() {
    // Create a Fluxion instance using our configuration, and give it the name `host`
    let system = Fluxion::<FluxionConfig>::new("host");
}

```

Here we simply define a new unit struct, and implement `Actor` for it. We need to provide an error type that our actor might return, however we can leave it `()` for now. We also need to use the `async_trait` crate until `async_fn_in_traits` is stabilized:

```sh
cargo add async_trait
```

## Defining Messages

We also need to define the messages that we are sending to actors. `()` implements `Message` by default, but we can also implement `Message` for our own structs:

```rust
mod config;

use config::FluxionConfig;

use fluxion::{Fluxion, Actor, FluxionParams, Message};

/// Our actor, which can contain any data we want. For now we will leave it empty.
struct MyActor;

#[async_trait::async_trait]
impl<C: FluxionParams> Actor<C> for MyActor {
    type Error = ();
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

Now that we have defined a message and an actor, we need to implement message handling logic for the actor. We will do this in the next page.