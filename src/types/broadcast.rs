//! Broadcast channel implementation used by notifications

use core::sync::atomic::{AtomicUsize, Ordering, AtomicBool};

use alloc::{collections::VecDeque, sync::Arc};
use maitake_sync::RwLock;


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
}

impl<T: Clone> Inner<T> {

    pub fn new() -> Self {
        Self {
            queue: RwLock::new(VecDeque::new()),
            head: 0.into(),
            wrapped: true.into(),
            observed_wrap: 0.into(),
            receivers: 0.into(),
        }
    }

    /// Pushes a message to the queue. Returns true if message was successfully pushed, false if not.
    pub async fn push(&mut self, message: T) -> bool {
        
        // If there are no receivers, return false
        if self.receivers.load(Ordering::Relaxed) == 0 {
            return false;
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

        true
    }

    /// Pop a message from the queue at the given position
    pub async fn pop(&self, tail: &mut usize) -> Option<T> {
        // If head equals tail, then no pushes have happened yet.
        let head = self.head.load(Ordering::Relaxed);
        if  head == *tail || head == 0 {
            return None;
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
            return None;
        };

        // Otherwise, fetch and increment the number of recievers that have seen this
        let seen = elem.1.fetch_add(1, Ordering::Relaxed);

        // If we just incremented the number of actors that have seen the message past/to the number of receivers,
        // and this is the last message, then pop this message, otherwise, clone it before returning
        if seen + 1 >= self.receivers.load(Ordering::Relaxed) && index + 1 >= queue.len() {

            // Relock the queue as write
            drop(queue);
            
            // Return None if this is None. It shouldn't be, though, because we didn't exit at the previous let Some.
            self.queue.write().await.pop_back().map(|v| v.0)
        } else {
            Some(elem.0.clone())
        }
    }
}

/// The Receive half of the channel
pub struct Receiver<T> {
    /// The wrapped inner value
    inner: Arc<Inner<T>>,
    /// The current tail
    tail: usize,
}

impl<T: Clone> Receiver<T> {

}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    pub async fn test_inner() {
        // Create a new inner
        let mut inner = Inner::<i32>::new();

        // Pushing with no receivers should return false
        // and keep the queue empty
        assert!(!inner.push(0).await);
        assert_eq!(inner.queue.read().await.len(), 0);

        // Now we simulate two receivers
        let mut r1_tail = 0;
        let mut r2_tail = 0;
        inner.receivers.store(2, Ordering::Relaxed);

        // Popping with no elements should return None, and should not increment either position
        assert_eq!(inner.pop(&mut r1_tail).await, None);
        assert_eq!(inner.pop(&mut r2_tail).await, None);
        assert_eq!(r1_tail, 0);
        assert_eq!(r2_tail, 0);

        // Now, because there are receivers, pushing should return true
        assert!(inner.push(1).await);
        assert!(inner.push(2).await);
        assert_eq!(inner.queue.read().await.len(), 2);

        // Both receivers should receive the first value
        // when called in order, and increment their counters.
        assert_eq!(inner.pop(&mut r1_tail).await, Some(1));
        assert_eq!(r1_tail, 1);
        assert_eq!(inner.pop(&mut r2_tail).await, Some(1));
        assert_eq!(r2_tail, 1);
        assert_eq!(inner.queue.read().await.len(), 1);
        
        // When executed out of order, the same should work.
        assert_eq!(inner.pop(&mut r2_tail).await, Some(2));
        assert_eq!(r2_tail, 2);
        assert_eq!(inner.pop(&mut r1_tail).await, Some(2));
        assert_eq!(r1_tail, 2);
        assert_eq!(inner.queue.read().await.len(), 0);

        // It should again return None, and no incrementation should happen
        assert_eq!(inner.pop(&mut r1_tail).await, None);
        assert_eq!(inner.pop(&mut r2_tail).await, None);
        assert_eq!(r1_tail, 2);
        assert_eq!(r2_tail, 2);

        // Now push two more values
        assert!(inner.push(3).await);
        assert!(inner.push(4).await);

        // And if both are done two at a time, it should not remove either until both others are done (but should increment the counters)
        assert_eq!(inner.pop(&mut r1_tail).await, Some(3));
        assert_eq!(inner.pop(&mut r1_tail).await, Some(4));
        // Popping this one again should return None and not increment
        assert_eq!(inner.pop(&mut r1_tail).await, None);
        assert_eq!(r1_tail, 4);
        // And the queue should have two entries
        assert_eq!(inner.queue.read().await.len(), 2);

        // Same for the other receiver, except the queue should no longer have any entries
        assert_eq!(inner.pop(&mut r2_tail).await, Some(3));
        assert_eq!(inner.pop(&mut r2_tail).await, Some(4));
        assert_eq!(r2_tail, 4);
        assert_eq!(inner.queue.read().await.len(), 0);

        // After all of this, head should be 4
        assert_eq!(inner.head.load(Ordering::Relaxed), 4);

        // Now, lets set head and both indexes to usize::MAX
        inner.head.store(usize::MAX, Ordering::Relaxed);
        r1_tail = usize::MAX;
        r2_tail = usize::MAX;

        // Now, assert that reading both does nothing
        assert_eq!(inner.pop(&mut r1_tail).await, None);
        assert_eq!(inner.pop(&mut r2_tail).await, None);

        // Assert that pushing one more works
        assert!(inner.push(5).await);
        
        // And that it wraps to one and sets the wrapped falg
        assert_eq!(inner.head.load(Ordering::Relaxed), 1);
        assert_eq!(inner.observed_wrap.load(Ordering::Relaxed), 0);
        assert!(inner.wrapped.load(Ordering::Relaxed));

        // Now, pop one more from one of the channels
        assert_eq!(inner.pop(&mut r1_tail).await, Some(5));
        // observed_wrap should have incremented, wrapped should still be true,
        // and the tail should be 1
        assert_eq!(inner.observed_wrap.load(Ordering::Relaxed), 1);
        assert!(inner.wrapped.load(Ordering::Relaxed));
        assert_eq!(r1_tail, 1);

        // Pop from the last channel
        assert_eq!(inner.pop(&mut r2_tail).await, Some(5));
        // And now, observed_wrap should have reset to zero, wrapped reset to false,
        // and tail should be 1
        assert_eq!(inner.observed_wrap.load(Ordering::Relaxed), 0);
        assert!(!inner.wrapped.load(Ordering::Relaxed));
        assert_eq!(r2_tail, 1);

        // And now head should be 1
        assert_eq!(inner.head.load(Ordering::Relaxed), 1);
    }
}