//! Broadcast channel implementation used by notifications


use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Waker, Poll};
use alloc::boxed::Box;
use maitake_sync::RwLock;
use alloc::{collections::VecDeque, vec::Vec, sync::Arc};

/// Creates a new broadcast channel
pub fn channel<T: Clone>() -> (Sender<T>, Receiver<T>) {
    // Create the inner and put it in an Arc
    let inner = Arc::new(Inner::<T>::new());

    // Get and return the sender and receiver
    (
        inner.sender(),
        inner.receiver()
    )
}


/// The internal representation of a message on the broadcast channel
struct BroadcastMessage<T: Clone> {
    /// The contents of the message
    message: T,
    /// The number of recievers who received the message
    received: AtomicUsize,
}

/// This struct is held via an `Arc` by both receivers and senders.
struct Inner<T: Clone> {
    /// The queue on which messages are stored
    queue: RwLock<VecDeque<BroadcastMessage<T>>>,
    /// The number of receivers registered with the channel
    receivers: AtomicUsize,
    /// The wakers currently registered
    wakers: RwLock<Vec<Waker>>
}


impl<T: Clone> Inner<T> {
    /// Create a new inner
    pub fn new() -> Self {
        Self {
            queue: RwLock::new(VecDeque::new()),
            receivers: AtomicUsize::new(0),
            wakers: RwLock::new(Vec::new()),
        }
    }

    /// Create a new [`Sender`] from an [`Arc<Inner>`]
    pub fn sender(self: &Arc<Self>) -> Sender<T> {
        Sender {
            inner: self.clone()
        }
    }

    /// Create a new [`Receiver`] from an [`Arc<inner>`]
    pub fn receiver(self: &Arc<Self>) -> Receiver<T> {
        self.receivers.fetch_add(1, Ordering::Relaxed);

        Receiver {
            waker: None,
            inner: self.clone(),
            recv_fut: None,
        }
    }
    
    /// Publish a message to the channel.
    /// Returns Err(message) if there are no receivers.
    pub async fn publish(&self, message: T) -> Result<(), T> {

        // If there are currently no receivers, then completely clear the queue and error
        if self.receivers.load(Ordering::Relaxed) == 0 {
            self.queue.write().await.clear();
            return Err(message);
        }

        // Otherwise, create a new broadcast channel, and push it to the deque
        let bm = BroadcastMessage {
            message,
            received: AtomicUsize::new(0) // No recievers have received the message yet
        };

        // Locking as write 
        let mut queue = self.queue.write().await;
        queue.push_back(bm);

        // Wake every waiting task
        while let Some(w) = self.wakers.write().await.pop() {
            w.wake();
        }

        Ok(())
    }

    /// Retrives the first value of the queue, decrements the received count,
    /// and removes the value if the count reaches zero. Returns None if the queue is empty,
    /// and Some(message) if a message is ready, cloning message.
    pub async fn try_recv(self: Arc<Self>) -> Option<T> {

        // Lock the queue as a reader
        let queue = self.queue.read().await;

        // Get a reference to the first element
        let first = queue.front()?;

        // Increment received counter
        let prev = first.received.fetch_add(1, Ordering::Relaxed);

        // If we are not the last to remove it, then just clone and return the first value
        if prev + 1 < self.receivers.load(Ordering::Relaxed) {
            Some(first.message.clone())
        } else {
            // If we are the last to read it, then re-lock the queue as a writer and pop from the front
            
            drop(queue); // Drop our read lock

            // Don't clone if we pop it.
            self.queue.write().await.pop_front().and_then(|v| Some(v.message))
        }
    }
}

/// The send side of a broadcast channel
#[derive(Clone)]
pub struct Sender<T: Clone> {
    inner: Arc<Inner<T>>
}

impl<T: Clone> Sender<T> {
    /// Send a message on the broadcast channel, returns Err(message)
    /// if there are no receivers.
    pub async fn send(&self, message: T) -> Result<(), T> {
        self.inner.publish(message).await
    }
}



/// The receive side of a broadcast channel
#[pin_project::pin_project(PinnedDrop)]
pub struct Receiver<T: Clone> {
    /// The waker used by the receive future
    waker: Option<Waker>,
    inner: Arc<Inner<T>>,
    /// The currently receiving future
    #[pin]
    recv_fut: Option<Pin<Box<dyn Future<Output = Option<T>>>>>
}

impl<T: Clone + 'static> Receiver<T> {

    /// Returns Some(message) if a message is ready, and None if not
    pub async fn try_recv(&self) -> Option<T> {
        self.inner.clone().try_recv().await
    }

    pub async fn recv(&mut self) -> T {
        self.await
    }
}

impl<T: Clone + 'static> Future for Receiver<T> {
    type Output = T;

    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        
        // If there is currently no try_recv future, create it
        if self.recv_fut.is_none() {
            self.recv_fut = Some(Box::pin(self.inner.clone().try_recv()));
        }

        let recv_fut = self.recv_fut.as_mut();
        
        // Get the next message
        let message: Option<T> = match recv_fut.unwrap().as_mut().poll(cx) {
            Poll::Ready(v) => v,
            Poll::Pending => {
                return Poll::Pending;
            }
        }; // Return pending if we lock.

        // If a message is ready, return it
        if let Some(m) = message {
            self.waker = None;
            Poll::Ready(m)
        } else {
            // Otherwise, lock wakers as write and push the waker
            let wakers = self.inner.wakers.try_write();

            // If we can't lock as write right now, then return Pending
            let Some(mut wakers) = wakers else {
                return Poll::Pending;
            };

            // Push the waker
            wakers.push(cx.waker().clone());

            Poll::Pending
        }

        
    }
}


#[pin_project::pinned_drop]
impl<T: Clone> PinnedDrop for Receiver<T> {
    fn drop(self: Pin<&mut Self>) {
        // Decrement the number of receivers on drop.
        self.inner.receivers.fetch_sub(1, Ordering::Relaxed);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_receive_one() {
        let (tx, mut rx) = channel::<i32>();

        // Make sure that a single value can be sent and received over the channel.
        tx.send(1).await;

        assert_eq!(rx.recv().await, 1);

        // Assert that several arrive in order
        tx.send(2).await;
        tx.send(3).await;
        tx.send(4).await;

        assert_eq!(rx.recv().await, 2);
        assert_eq!(rx.recv().await, 3);
        assert_eq!(rx.recv().await, 4);
    }


}