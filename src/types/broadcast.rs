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

enum Offset {
    Offset(usize),
    Free(usize),
}

/// This allocates and retrieves offset for identifiers, and allows every offset to be quickly decremented.
/// or incremented.
struct OffsetAllocator {
    /// The actual offsets
    offsets: Vec<Offset>,
    /// A global offset which is *subtracted* from every other offset
    global_offset: usize,
    /// The next offset that is free
    next_free: usize,
}

impl OffsetAllocator {
    pub fn new() -> Self {
        Self {
            offsets: Vec::new(),
            global_offset: 0,
            next_free: 0,
        }
    }

    /// Allocates a new identifier with the given offset
    pub fn alloc(&mut self, offset: usize) -> usize {
        // The new id is going to be the current value of next_free
        let id = self.next_free;

        // Get the value of the current id, and if it is Some, update next_free and store accordingly
        if let Some(v) = self.offsets.get_mut(id) {
            // If it is free, retrive the next free space and update it.
            // if it is not free, something has gone terribly wrong.
            if let Offset::Free(next_free) = v {
                self.next_free = *next_free;
            } else {
                // In this case, just push the element onto the vector and update next_free to be the
                // vector's length
                self.offsets.push(Offset::Offset(offset));
                self.next_free = self.offsets.len();
                return self.offsets.len() - 1;
            }
            // Update the offset and return id
            *v = Offset::Offset(offset);
            id
        } else {
            // In this case, we just push and increment offset
            self.offsets.push(Offset::Offset(offset));
            self.next_free = self.offsets.len();
            self.offsets.len() - 1
        }
    }

    /// Deletes an offset
    pub fn free(&mut self, id: usize) {
        // If the offset exists
        if let Some(v) = self.offsets.get_mut(id) {
            // And it is allocated
            if let Offset::Free(_) = v {
                return;
            }

            // Then replace it with a free value containing the next free value
            *v = Offset::Free(self.next_free);

            // And update next_free accordingly
            self.next_free = id;
        }
    }

    /// Gets an offset
    pub fn get(&self, id: usize) -> Option {
        self.offsets.get(id).and_then(|v| match v {
            Offset::Free(_) => None,
            Offset::Offset(v) => Some(v)
        })
    }

    /// Add to an offset
    pub fn add(&self, id: usize, num: usize) {
        self.offsets.get_mut(id).and_then(|v| {
            if let Offset::Offset(v) = v {
                *v += num;
            }
            Some(())
        });
    }
}


/// The internal representation of a message on the broadcast channel
#[derive(Debug)]
struct BroadcastMessage<T: Clone> {
    /// The contents of the message
    message: T,
    /// The number of recievers who received the message
    received: AtomicUsize,
}

/// This struct is held via an `Arc` by both receivers and senders.
#[derive(Debug)]
struct Inner<T: Clone> {
    /// The queue on which messages are stored
    queue: RwLock<VecDeque<BroadcastMessage<T>>>,
    /// The number of receivers registered with the channel
    receivers: AtomicUsize,
    /// The wakers currently registered
    wakers: RwLock<Vec<Waker>>,
    /// The offsets of each receiver
    offsets: RwLock<OffsetAllocator>,
}


impl<T: Clone> Inner<T> {
    /// Create a new inner
    pub fn new() -> Self {
        Self {
            queue: RwLock::new(VecDeque::new()),
            receivers: AtomicUsize::new(0),
            wakers: RwLock::new(Vec::new()),
            offsets: RwLock::new(OffsetAllocator::new())
        }
    }

    /// Create a new [`Sender`] from an [`Arc<Inner>`]
    pub fn sender(self: &Arc<Self>) -> Sender<T> {
        Sender {
            inner: self.clone()
        }
    }

    /// Create a new [`Receiver`] from an [`Arc<inner>`]
    pub async fn receiver(self: &Arc<Self>) -> Receiver<T> {
        self.receivers.fetch_add(1, Ordering::Relaxed);

        Receiver {
            inner: self.clone()
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
pub struct Receiver<T: Clone> {
    inner: Arc<Inner<T>>,
}

impl<T: Clone + 'static> Receiver<T> {

    /// Returns Some(message) if a message is ready, and None if not
    pub async fn try_recv(&self) -> Option<T> {
        self.inner.clone().try_recv().await
    }

    pub async fn recv(&self) -> T {
        let mut recv = ReceiveFuture {
            inner: self.inner.clone(),
            recv_fut: None,
        };
        let fut = Pin::new(&mut recv);

        fut.await
    }
}

impl<T: Clone> Drop for Receiver<T> {
    fn drop(&mut self) {
        // Decrement the number of receivers on drop.
        self.inner.receivers.fetch_sub(1, Ordering::Relaxed);
    }
}
impl<T: Clone> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        
        self.inner.receivers.fetch_add(1, Ordering::Relaxed);

        Self { inner: self.inner.clone()}
    }
}


#[pin_project::pin_project]
struct ReceiveFuture<T: Clone> {
    /// A reference to the inner struct
    inner: Arc<Inner<T>>,
    /// The pinned future (Inner::try_recv) that we are currently waiting on
    #[pin]
    recv_fut: Option<Pin<Box<dyn Future<Output = Option<T>>>>>,
}

impl<T: Clone + 'static> Future for ReceiveFuture<T> {
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





#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[tokio::test]
    async fn test_receive_one() {
        let (tx, rx) = channel::<i32>();

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

    #[tokio::test]
    async fn test_receive_many() {
        
        let (tx, rx1) = channel::<i32>();
        
        // Clone the receiver
        let rx2 = rx1.clone();
        let rx3 = rx1.clone();
        
        // Send a single value
        tx.send(1).await;

        // Make sure that every receiver recieves said value
        
        assert_eq!(rx1.recv().await, 1);
        assert_eq!(rx2.recv().await, 1);
        assert_eq!(rx3.recv().await, 1);

        // Send a couple values
        tx.send(2).await;
        tx.send(3).await;

        // Ensure each receiver recieves the values
        assert_eq!(rx1.recv().await, 2);
        assert_eq!(rx2.recv().await, 2);
        assert_eq!(rx3.recv().await, 2);
        assert_eq!(rx1.recv().await, 3);
        assert_eq!(rx2.recv().await, 3);
        assert_eq!(rx3.recv().await, 3);

        // Send two more
        tx.send(4).await;
        tx.send(5).await;

        // Ensure each receiver may receive out of order
        assert_eq!(rx1.recv().await, 4);
        assert_eq!(rx1.recv().await, 5);
        assert_eq!(rx2.recv().await, 4);
        assert_eq!(rx2.recv().await, 5);
        assert_eq!(rx3.recv().await, 4);
        assert_eq!(rx3.recv().await, 5);
    }


}