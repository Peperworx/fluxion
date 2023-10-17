use fluxion::{Executor, FluxionParams};




struct TokioExecutor;

impl Executor for TokioExecutor {
    fn spawn<T>(&self, future: T) -> fluxion::types::executor::JoinHandle<T::Output>
    where
        T: std::future::Future + Send + 'static,
        T::Output: Send + 'static {
        let handle = tokio::spawn(future);
        fluxion::types::executor::JoinHandle { handle: Box::pin(async {
            handle.await.unwrap()
        }) }
    }
}

#[derive(Clone)]
struct SystemConfig;
impl FluxionParams for SystemConfig {
    type Executor = TokioExecutor;
}

#[tokio::main(flavor = "current_thread")]
async fn main() {

}