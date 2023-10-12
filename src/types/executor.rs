//! Provides an [`Executor`] trait which is used to allow spawning async tasks without depending on a
//! specific executor.


use core::future::Future;
use core::pin::Pin;
use alloc::boxed::Box;

/// # [`Executor`]
/// Enables spawning async tasks in an executor agnostic manner.
pub trait Executor: Send + Sync + 'static {
    /// Spawn an async task.
    fn spawn<T>(&self, future: T) -> JoinHandle<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static;
}

/// # [`JoinHandle`]
/// Executor-agnostic `JoinHandle`.
pub struct JoinHandle<T> {
    pub handle: Pin<Box<dyn Future<Output = T> + Send + 'static>>
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        self.handle.as_mut().poll(cx)
    }
}