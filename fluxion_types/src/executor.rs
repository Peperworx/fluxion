//! # Executor-Agnostic utilities.
//! Fluxion requires the ability to spawn async tasks. This module provides a trait, [`Executor`],
//! which may be implemented for various different async executors, enabling Fluxion to operate in an executor-agnostic manner.

use core::future::Future;
use core::pin::Pin;
use alloc::boxed::Box;


pub trait Executor: Send + Sync + 'static {
    /// Spawn an async task.
    fn spawn<T>(future: T) -> JoinHandle<T::Output>
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
        Pin::new(&mut self.handle).poll(cx)
    }
}