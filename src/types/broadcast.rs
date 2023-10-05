//! Broadcast channel implementation used by notifications

use core::{sync::atomic::{AtomicUsize, Ordering, AtomicBool}, future::Future, task::{Poll, Waker}, pin::Pin};

use alloc::{collections::VecDeque, sync::Arc, boxed::Box, vec::Vec};
use maitake_sync::RwLock;

/// Creates a new sender receiver pair
pub async fn channel(capacity: usize) -> (Sender<T>, Receiver<T>) {
    // Create the inner
    let inner = Arc::new(Inner::new(Some(capacity)));

    (
        inner.sender(),
        inner.receiver().await
    )
}


/// An Error returned when receiving from the channel.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub enum TryRecvError {
    /// Set if the current receiver is lagged, and contains how many messages the receiver has missed.
    Lagged(usize),
    /// The queue is empty
    Empty
}

/// This struct is held via an `Arc` by both receivers and senders.
#[derive(Debug)]
struct Inner<T> {
    /// The queue stores individual messages, and the number of receivers that have seen it.
    queue: RwLock<VecDeque<(T, AtomicUsize)>>,
    /// The head index of the queue. This value will wrap after usize::MAX messages.
    /// While this should not happen before an application is restarted, there is still a failsafe.
    head: AtomicUsize,
    /// When head wraps, this is set
    wrapped: AtomicBool,
    /// When wrapped is set, this value is incremented by every reciever that has acknowledged the wrap
    /// and updated its tail. If every receiver does not update this by the time head wraps again, then receivers
    /// that have not acknowledged will miss about 18,446,744,073,709,551,615 messages, and the user of the library
    /// should reconsider their life choices.
    observed_wrap: AtomicUsize,
    /// The number of receivers
    receivers: AtomicUsize,
    /// The wakers for every receiver waiting for a value
    receive_ops: RwLock<Vec<Waker>>,
    /// The optional bound of the queue. Values after this will be dropped.
    bound: Option<usize>,
}

impl<T: Clone> Inner<T> {

    pub fn new(bound: Option<usize>) -> Self {
        Self {
            queue: RwLock::new(VecDeque::new()),
            head: 0.into(),
            wrapped: true.into(),
            observed_wrap: 0.into(),
            receivers: 0.into(),
            receive_ops: RwLock::new(Vec::new()),
            bound
        }
    }

    /// Creates a new sender associated with this Inner
    pub fn sender(self: &Arc<Self>) -> Sender<T> {
        Sender {
            inner: self.clone()
        }
    }

    /// Creates a new reciever associated with this Inner
    pub async fn receiver(self: &Arc<Self>) -> Receiver<T> {
        // Increment the reciever count
        self.receivers.fetch_add(1, Ordering::Relaxed);

        // Lock the queue
        let queue = self.queue.read().await;

        let offset = if let Some(bound) = self.bound {
            queue.len().min(bound)
        } else {
            queue.len()
        };

        // Create and retrieve the receiver
        Receiver {
            inner: self.clone(),
            tail: self.as_ref().head.load(Ordering::Relaxed) - offset
        }
    }
    

    /// Pushes a message to the queue. Returns true if message was successfully pushed, false if not.
    pub async fn push(&self, message: T) -> Result<(), T> {
        
        // If there are no receivers, return false
        if self.receivers.load(Ordering::Relaxed) == 0 {
            return Err(message);
        }

        // If this would cause head to wrap, then set the wrap flag to true and reset head to one
        if self.head.compare_exchange(usize::MAX, 1, Ordering::Relaxed, Ordering::Relaxed) == Ok(usize::MAX) {
            self.wrapped.store(true, Ordering::Relaxed);
            self.observed_wrap.store(0, Ordering::Relaxed);
        } else {
            // Otherwise, just increment head
            self.head.fetch_add(1, Ordering::Relaxed);
        }

        // Push the value to the front of the queue
        self.queue.write().await.push_front((message, AtomicUsize::new(0)));

        Ok(())
    }

    /// Pop a message from the queue at the given position
    pub async fn pop(&self, tail: &mut usize) -> Result<T, TryRecvError> {
        // If head equals tail, then no pushes have happened yet.
        let head = self.head.load(Ordering::Relaxed);
        if  head == *tail || head == 0 {
            return Err(TryRecvError::Empty);
        }

        // Because items are pushed at the front of the queue, the first element is index `head`.
        // The next is `head - 1`, and so on. This means that the actual index is `head - tail`.

        // We also worry about wrapping here. If wrapped is true, and the tail is larger than head, set tail to 0
        // And acknowledge the wrap
        if self.wrapped.load(Ordering::Relaxed) && head < *tail {
            *tail = 0;
            let observed_wrap = self.observed_wrap.fetch_add(1, Ordering::Relaxed) + 1;

            // If observed_wrap is larger than or equal the number of receivers, then we can safely reset the wrapped flag
            if observed_wrap >= self.receivers.load(Ordering::Relaxed) {
                self.wrapped.store(false, Ordering::Relaxed);
                self.observed_wrap.store(0, Ordering::Relaxed);
            }
        }

        
        
        
        // Now, lets get the first actual index
        // The only reason why tail would be larger than head was if we wrapped. And we already verified that this is true here.
        // Head will also always be 1 or more, so we need to subtract 1 to make it into an index.
        let index = head - 1 - *tail;

        // If the index is larger than or equal to the bound, then return that we have lagged
        if let Some(bound) = self.bound  {
            if index >= bound {
                // Get how much we lagged by
                let lag = (head-*tail)-bound;

                // If we lag, update tail to catch us up
                *tail += lag;

                // And truncate the queue to the bound
                self.queue.write().await.truncate(bound);

                return Err(TryRecvError::Lagged(lag));
            }
        }

        // Increment tail, if such an incrementation would not push it past head
        if *tail < head {
            *tail += 1;
        }

        // Lock the queue as read
        let queue = self.queue.read().await;
        
        // Get the element
        let elem = queue.get(index);

        // If we got None, return None
        let Some(elem) = elem else {
            return Err(TryRecvError::Empty);
        };

        // Otherwise, fetch and increment the number of recievers that have seen this
        let seen = elem.1.fetch_add(1, Ordering::Relaxed);

        // If we just incremented the number of actors that have seen the message past/to the number of receivers,
        // and this is the last message, then pop this message, otherwise, clone it before returning
        let m = if seen + 1 >= self.receivers.load(Ordering::Relaxed) && index + 1 >= queue.len() {

            // Relock the queue as write
            drop(queue);
            
            // Return None if this is None. It shouldn't be, though, because we didn't exit at the previous let Some.
            self.queue.write().await.pop_back().map(|v| v.0).ok_or(TryRecvError::Empty)
        } else {
            let m = elem.0.clone();

            // Allow us to lock the queue again later
            drop(queue);

            Ok(m)
        };
        

        // If the length of the queue exceeds the bound, pop from the back until it is within bounds
        if let Some(bound) = self.bound {
            if self.queue.read().await.len() > bound {
                // Lock as write
                let mut queue = self.queue.write().await;

                // Truncate the queue to the bound
                queue.truncate(bound);
            }
        }

        m
    }

    /// Tries to receive over the queue, returning a tuple containing an Optional message,
    /// which will be None if no messages are ready, and a new value for the tail of the current receiver
    pub async fn try_recv(self: Arc<Self>, mut tail: usize) -> (Result<T, TryRecvError>, usize) {
        // Just pop from the queue and relay tail
        (self.pop(&mut tail).await, tail)
    }
}

/// The Receive half of the channel
pub struct Receiver<T> {
    /// The wrapped inner value
    inner: Arc<Inner<T>>,
    /// The current tail
    tail: usize,
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        // Increment the receiver count
        self.inner.receivers.fetch_add(1, Ordering::Relaxed);

        Self { inner: self.inner.clone(), tail: self.tail.clone() }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        // Decrement the receiver count
        self.inner.receivers.fetch_sub(1, Ordering::Relaxed);
    }
}

impl<T: Clone + 'static> Receiver<T> {
    /// Receive from the channel, returns None if the receiver lags.
    pub async fn recv(&mut self) -> Option<T> {
        let fut = ReceiveFut {
            inner: self.inner.clone(),
            tail: &mut self.tail,
            recv_fut: None,
        };

        fut.await
    }

    /// Try to recieve from the channel.
    /// 
    /// # Errors
    /// Errors if the receiver lags behind the channel's bound, or if there are no messages left in the channel
    pub async fn try_recv(&mut self) -> Result<T, TryRecvError> {
        // Try recv
        self.inner.pop(&mut self.tail).await
    }
}

type BoxedFuture<T> = Pin<Box<dyn Future<Output = T>>>;

#[pin_project::pin_project]
struct ReceiveFut<'a, T> {
    /// The wrapped inner value
    inner: Arc<Inner<T>>,
    /// A mutable reference to the tail
    tail: &'a mut usize,
    /// The pinned future (Inner::pop) that we are currently waiting on
    #[pin]
    recv_fut: Option<BoxedFuture<(Result<T, TryRecvError>, usize)>>,
}

impl<'a, T: Clone + 'static> Future for ReceiveFut<'a, T> {
    type Output = Option<T>;

    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        
        // If there is currently no recv future, create it
        if self.recv_fut.is_none() {
            self.recv_fut = Some(Box::pin(self.inner.clone().try_recv(*self.tail)));
        }

        // Get the next message
        let (message, tail) = match self.recv_fut.as_mut().unwrap().as_mut().poll(cx) {
            Poll::Ready(v) => v,
            Poll::Pending => {
                return Poll::Pending;
            }
        };

        // Update tail
        *self.tail = tail;

        // If a message is ready, return it
        if let Ok(m) = message {
            Poll::Ready(Some(m))
        } else {
            if let Err(TryRecvError::Lagged(_)) = message {
                // We lagged, so return None.
                return Poll::Ready(None);
            }

            // Otherwise, lock the wakers list as write and push out waker
            let wakers = self.inner.receive_ops.try_write();

            // If we successfully lock it, push the waker
            if let Some(mut wakers) = wakers {
                wakers.push(cx.waker().clone());
            } else {
                // If we can't lock it right now, schedule this future to be
                // woken again.
                cx.waker().wake_by_ref();
            }

            // And return pending. 
            Poll::Pending
        }
    }
}


/// The send half of a channel
#[derive(Clone)]
pub struct Sender<T> {
    /// The wrapped inner
    inner: Arc<Inner<T>>,
}

impl<T: Clone> Sender<T> {
    /// Send a message on the channel, returning Err(message)
    /// if there are no receivers to send the message to
    pub async fn send(&self, message: T) -> Result<(), T> {
        self.inner.push(message).await
    }

    /// Gets a receiver for this channel
    pub async fn subscribe(&self) -> Receiver<T> {
        self.inner.receiver().await
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    pub async fn test_inner() {
        // Create a new inner
        let inner = Inner::<i32>::new(Some(2));

        // Pushing with no receivers should return false
        // and keep the queue empty
        assert_eq!(inner.push(0).await, Err(0));
        assert_eq!(inner.queue.read().await.len(), 0);

        // Now we simulate two receivers
        let mut r1_tail = 0;
        let mut r2_tail = 0;
        inner.receivers.store(2, Ordering::Relaxed);

        // Popping with no elements should return None, and should not increment either position
        assert_eq!(inner.pop(&mut r1_tail).await, Err(TryRecvError::Empty));
        assert_eq!(inner.pop(&mut r2_tail).await, Err(TryRecvError::Empty));
        assert_eq!(r1_tail, 0);
        assert_eq!(r2_tail, 0);

        // Now, because there are receivers, pushing should return true
        assert_eq!(inner.push(1).await, Ok(()));
        assert_eq!(inner.push(2).await, Ok(()));
        assert_eq!(inner.queue.read().await.len(), 2);

        // Both receivers should receive the first value
        // when called in order, and increment their counters.
        assert_eq!(inner.pop(&mut r1_tail).await, Ok(1));
        assert_eq!(r1_tail, 1);
        assert_eq!(inner.pop(&mut r2_tail).await, Ok(1));
        assert_eq!(r2_tail, 1);
        assert_eq!(inner.queue.read().await.len(), 1);
        
        // When executed out of order, the same should work.
        assert_eq!(inner.pop(&mut r2_tail).await, Ok(2));
        assert_eq!(r2_tail, 2);
        assert_eq!(inner.pop(&mut r1_tail).await, Ok(2));
        assert_eq!(r1_tail, 2);
        assert_eq!(inner.queue.read().await.len(), 0);

        // It should again return None, and no incrementation should happen
        assert_eq!(inner.pop(&mut r1_tail).await, Err(TryRecvError::Empty));
        assert_eq!(inner.pop(&mut r2_tail).await, Err(TryRecvError::Empty));
        assert_eq!(r1_tail, 2);
        assert_eq!(r2_tail, 2);

        // Now push two more values
        assert_eq!(inner.push(3).await, Ok(()));
        assert_eq!(inner.push(4).await, Ok(()));

        // And if both are done two at a time, it should not remove either until both others are done (but should increment the counters)
        assert_eq!(inner.pop(&mut r1_tail).await, Ok(3));
        assert_eq!(inner.pop(&mut r1_tail).await, Ok(4));
        // Popping this one again should return None and not increment
        assert_eq!(inner.pop(&mut r1_tail).await, Err(TryRecvError::Empty));
        assert_eq!(r1_tail, 4);
        // And the queue should have two entries
        assert_eq!(inner.queue.read().await.len(), 2);

        // Same for the other receiver, except the queue should no longer have any entries
        assert_eq!(inner.pop(&mut r2_tail).await, Ok(3));
        assert_eq!(inner.pop(&mut r2_tail).await, Ok(4));
        assert_eq!(r2_tail, 4);
        assert_eq!(inner.queue.read().await.len(), 0);

        // After all of this, head should be 4
        assert_eq!(inner.head.load(Ordering::Relaxed), 4);

        // Now, lets set head and both indexes to usize::MAX
        inner.head.store(usize::MAX, Ordering::Relaxed);
        r1_tail = usize::MAX;
        r2_tail = usize::MAX;

        // Now, assert that reading both does nothing
        assert_eq!(inner.pop(&mut r1_tail).await, Err(TryRecvError::Empty));
        assert_eq!(inner.pop(&mut r2_tail).await, Err(TryRecvError::Empty));

        // Assert that pushing one more works
        assert_eq!(inner.push(5).await, Ok(()));
        
        // And that it wraps to one and sets the wrapped falg
        assert_eq!(inner.head.load(Ordering::Relaxed), 1);
        assert_eq!(inner.observed_wrap.load(Ordering::Relaxed), 0);
        assert!(inner.wrapped.load(Ordering::Relaxed));

        // Now, pop one more from one of the channels
        assert_eq!(inner.pop(&mut r1_tail).await, Ok(5));
        // observed_wrap should have incremented, wrapped should still be true,
        // and the tail should be 1
        assert_eq!(inner.observed_wrap.load(Ordering::Relaxed), 1);
        assert!(inner.wrapped.load(Ordering::Relaxed));
        assert_eq!(r1_tail, 1);

        // Pop from the last channel
        assert_eq!(inner.pop(&mut r2_tail).await, Ok(5));
        // And now, observed_wrap should have reset to zero, wrapped reset to false,
        // and tail should be 1
        assert_eq!(inner.observed_wrap.load(Ordering::Relaxed), 0);
        assert!(!inner.wrapped.load(Ordering::Relaxed));
        assert_eq!(r2_tail, 1);

        // And now head should be 1
        assert_eq!(inner.head.load(Ordering::Relaxed), 1);

        // Now lets test lagging.

        // Push three values
        assert_eq!(inner.push(6).await, Ok(()));
        assert_eq!(inner.push(7).await, Ok(()));
        assert_eq!(inner.push(8).await, Ok(()));



        // This highlights an interesting fact about the queue:
        // Senders never wait for room in the queue. This may be changed in the future,
        // however this works fine for now.
        assert_eq!(inner.queue.read().await.len(), 3);

        // Here, the value 6 is lagged. We will see that we lagged by 1 if we try to receive
        assert_eq!(inner.pop(&mut r1_tail).await, Err(TryRecvError::Lagged(1)));
        // This should reset r1_tail to the bound, so we can check that here
        assert_eq!(r1_tail, inner.head.load(Ordering::Relaxed)-2);
        // It should also remove any messages that are out of bounds
        assert_eq!(inner.queue.read().await.len(), 2);
        assert_eq!(inner.pop(&mut r1_tail).await, Ok(7));

        // If we force three elements into the queue (one partially accessed by r1, and two more untouched)
        // then the length should be back to 3
        assert_eq!(inner.push(9).await, Ok(()));
        assert_eq!(inner.queue.read().await.len(), 3);

        // And a successful read from r1 should also truncate to the bound
        assert_eq!(inner.pop(&mut r1_tail).await, Ok(8));
        assert_eq!(inner.queue.read().await.len(), 2);

        // And now r2 should be lagged, but by two
        assert_eq!(inner.pop(&mut r2_tail).await, Err(TryRecvError::Lagged(2)));
        // And should recover properly
        assert_eq!(r2_tail, inner.head.load(Ordering::Relaxed)-2);
        assert_eq!(inner.pop(&mut r2_tail).await, Ok(8));

        // And when we should be able to read normally from both
        assert_eq!(inner.pop(&mut r1_tail).await, Ok(9));
        assert_eq!(inner.pop(&mut r2_tail).await, Ok(9));

    }

    #[tokio::test]
    pub async fn test_receiver() {
        // Create a new inner, and store it in an Arc
        let inner = Arc::new(Inner::<i32>::new(Some(2)));

        // Get a receiver from the inner
        let mut receiver = inner.receiver().await;

        // And get a reference to make our life easier
        let inner_ref = inner.as_ref();

        // Receiver count should be 1, and the receiver should start with a tail of
        // 0
        assert_eq!(inner_ref.receivers.load(Ordering::Relaxed), 1);
        assert_eq!(receiver.tail, 0);

        // If the receiver receives now, it would block forever, so let's push a message
        // This should return true because there is a receiver
        assert_eq!(inner.push(0).await, Ok(()));

        // Now, the receiver should return 0
        assert_eq!(receiver.recv().await, Some(0));
        // And its tail should be incremented
        assert_eq!(receiver.tail, 1);

        // This should also work with multiple messages
        assert_eq!(inner.push(1).await, Ok(()));
        assert_eq!(inner.push(2).await, Ok(()));
        assert_eq!(receiver.recv().await, Some(1));
        assert_eq!(receiver.recv().await, Some(2));
        assert_eq!(receiver.tail, 3);

        // Now if we force a bound overflow, the receiver should relay errors properly
        assert_eq!(inner.push(3).await, Ok(()));
        assert_eq!(inner.push(4).await, Ok(()));
        assert_eq!(inner.push(5).await, Ok(()));

        assert_eq!(receiver.try_recv().await, Err(TryRecvError::Lagged(1)));

        // And if we overflow again, then receiving without try_recv should return None
        assert_eq!(inner.push(6).await, Ok(()));
        assert_eq!(receiver.recv().await, None);

        // Now if we create a new receiver, it should have its tail positioned at the oldest available message
        let mut receiver2 = inner.receiver().await;
        // Which is just the same as receiver 2
        assert_eq!(receiver.tail, receiver2.tail); 

        // Now both should receive both new messages
        assert_eq!(receiver.recv().await, Some(5));
        assert_eq!(receiver.try_recv().await, Ok(6));
        assert_eq!(receiver2.recv().await, Some(5));
        assert_eq!(receiver2.try_recv().await, Ok(6));

        // And make sure that we have caught up
        assert_eq!(receiver.tail, inner.head.load(Ordering::Relaxed));
        assert_eq!(receiver2.tail, inner.head.load(Ordering::Relaxed));
        assert_eq!(inner.queue.read().await.len(), 0);

        // And when both receivers are dropped, receiver count zeroes out
        drop(receiver);
        assert_eq!(inner.receivers.load(Ordering::Relaxed), 1);
        drop(receiver2);
        assert_eq!(inner.receivers.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    pub async fn test_sender_receiver() {
        // Create an inner, bound of 2
        let inner = Arc::new(Inner::<i32>::new(Some(2)));

        // Create a sender
        let sender = inner.sender();

        // If we try to send a message now, we will fail because there are no receivers
        assert_eq!(sender.send(0).await, Err(0));

        // Now lets create a receiver from the sender
        let mut receiver = sender.subscribe().await;
        // And clone a second receiver
        let mut receiver2 = receiver.clone();

        // And lets clone a second sender too
        let sender2 = sender.clone();

        // If we send something now, both receivers should get it
        assert_eq!(sender.send(0).await, Ok(()));
        assert_eq!(receiver.recv().await, Some(0));
        assert_eq!(receiver2.recv().await, Some(0));

        // Same with the other sender
        assert_eq!(sender2.send(1).await, Ok(()));
        assert_eq!(receiver.recv().await, Some(1));
        assert_eq!(receiver2.recv().await, Some(1));
    }
}