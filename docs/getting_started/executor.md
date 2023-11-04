---
title: Defining an Executor
---

Remember that `TokioExecutor` struct we were using before?
You may have guessed that we would be using `tokio` as an example executor, and you would be correct.
Lets add that now, and just add the full feature set:

``` sh
cargo add tokio --features full
```

And now lets define a new unit struct that uses the global executor:

``` rust
use fluxion::Executor;

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
```

The above code is pretty simple. We implement the `Executor` trait on our unit struct, and in the spawn function, spawn a task on the global executor and return it's join handle, nicely wrapped in a common interface.

At this point, all of our code should look like this:

```rust
use fluxion::{FluxionParams, Executor};

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

On the next page, we will implementing a unit struct that uses bincode for serialization.