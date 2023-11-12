---
title: Defining a Message Serializer
---

We will be using bincode for our serializer. Lets add that real quick:

```sh
cargo add bincode serde
```

And lets define our message serializer struct:

```rust
use serde::{Serialize, Deserialize};

use fluxion::MessageSerializer;

/// Define our serializer unit type
struct BincodeSerializer;

// And implement the serialization functions for it.
impl MessageSerializer for BincodeSerializer {
    fn deserialize<T: for<'a> Deserialize<'a>>(message: Vec<u8>) -> Option<T> {
        bincode::deserialize(&message).ok()
    }

    fn serialize<T: Serialize>(message: T) -> Option<Vec<u8>> {
        bincode::serialize(&message).ok()
    }
}
```

As you can see here, the `MessageSerializer` trait only requires two functions: one for serializing and another for deserializing. These should both return `None` in case of a failure.

This should be our code now:

```rust
use serde::{Serialize, Deserialize};


use fluxion::{FluxionParams, Executor, MessageSerializer};

/// Define an executor to use
struct TokioExecutor;

/// Implement the executor trait
impl Executor for TokioExecutor {
    /// We only need to implement a single spawn function.
    fn spawn<T>(future: T) -> fluxion::types::executor::JoinHandle<T::Output>
    where
        T: std::future::Future + Send + 'static,
        T::Output: Send + 'static {

        // Spawn on tokio's global executor
        let handle = tokio::spawn(future);

        // Create an executor-agnostic join handle that waits for the task
        // to complete, and return it.
        fluxion::types::executor::JoinHandle { handle: Box::pin(async {
            handle.await.unwrap()
        }) }
    }
}

/// Define our serializer unit type
struct BincodeSerializer;

// And implement the serialization functions for it.
impl MessageSerializer for BincodeSerializer {
    fn deserialize<T: for<'a> Deserialize<'a>>(message: Vec<u8>) -> Option<T> {
        bincode::deserialize(&message).ok()
    }

    fn serialize<T: Serialize>(message: T) -> Option<Vec<u8>> {
        bincode::serialize(&message).ok()
    }
}

/// Our Fluxion instance's configuration parameters
struct FluxionConfig;

/// The actual configuration parameters
impl FluxionParams for FluxionConfig {
    /// The async executor that Fluxion should use
    type Executor = TokioExecutor;

    /// The serialization mechanism that Fluxion should use for foreign messages.
    type Serializer = BincodeSerializer;
}
```

Because this is so much code, we are going to move it into the file `config.rs`, and add a module declaration to our `main.rs` file:

```rust

mod config;

fn main() {
    println!("Hello, World!")
}

```