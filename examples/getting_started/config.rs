use serde::{Serialize, Deserialize};


use fluxion::{FluxionParams, Executor, MessageSerializer};

/// Define an executor to use
pub struct TokioExecutor;

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
pub struct BincodeSerializer;

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
pub struct FluxionConfig;

/// The actual configuration parameters
impl FluxionParams for FluxionConfig {
    /// The async executor that Fluxion should use
    type Executor = TokioExecutor;

    /// The serialization mechanism that Fluxion should use for foreign messages.
    type Serializer = BincodeSerializer;
}