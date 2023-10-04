//! Broadcast channel implementation used by notifications

use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::collections::VecDeque;


/// This struct is held via an `Arc` by both receivers and senders.
#[derive(Debug)]
struct Inner<T: Clone> {
    /// The queue stores individual messages, and the number of receivers that have seen it.
    queue: VecDeque<(T, AtomicUsize)>,
    /// The head index of the queue. This value will wrap after usize::MAX messages.
    /// While this should not happen before an application is restarted, there is still a failsafe.
    head: usize,
    /// When head wraps, this is set
    wrapped: bool,
    /// When wrapped is set, this value is incremented by every reciever that has acknowledged the wrap
    /// and updated its tail. If every receiver does not update this by the time head wraps again, then receivers
    /// that have not acknowledged will miss about 18,446,744,073,709,551,615 messages, and the user of the library
    /// should reconsider their life choices.
    observed_wrap: usize,
    /// The number of receivers
    receivers: usize,
}

impl<T: Clone> Inner<T> {

    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            head: 0,
            wrapped: true,
            observed_wrap: 0,
            receivers: 0,
        }
    }

    /// Pushes a message to the queue. Returns true if message was successfully pushed, false if not.
    pub fn push(&mut self, message: T) -> bool {
        
        // If there are no receivers, return false
        if self.receivers == 0 {
            return false;
        }

        // If this would cause head to wrap, then set the wrap flag to true and reset head to zero
        if self.head == usize::MAX {
            self.head = 1;
            self.wrapped = true;
            self.observed_wrap = 0;
        } else {
            // Otherwise, just increment head
            self.head += 1;
        }

        // Push the value to the front of the queue
        self.queue.push_front((message, AtomicUsize::new(0)));

        true
    }

    /// Pop a message from the queue at the given position
    pub fn pop(&mut self, tail: &mut usize) -> Option<T> {
        // If head equals tail, then no pushes have happened yet.
        if self.head == *tail || self.head == 0 {
            return None;
        }

        // Because items are pushed at the front of the queue, the first element is index `head`.
        // The next is `head - 1`, and so on. This means that the actual index is `head - tail`.

        // We also worry about wrapping here. If wrapped is true, and the tail is larger than head, set tail to 0
        // And acknowledge the wrap
        if self.wrapped && self.head < *tail {
            *tail = 0;
            self.observed_wrap += 1;

            // If observed_wrap is larger than or equal the number of receivers, then we can safely reset the wrapped flag
            if self.observed_wrap >= self.receivers {
                self.wrapped = false;
                self.observed_wrap = 0;
            }
        }
        
        
        // Now, lets get the first actual index
        // The only reason why tail would be larger than head was if we wrapped. And we already verified that this is true here.
        // Head will also always be 1 or more, so we need to subtract 1 to make it into an index.
        let index = self.head - 1 - *tail;

        // Increment tail, if such an incrementation would not push it past head
        if *tail < self.head {
            *tail += 1;
        }        

        // Get the element
        let elem = self.queue.get(index);

        // If we got None, return None
        let Some(elem) = elem else {
            return None;
        };

        // Otherwise, fetch and increment the number of recievers that have seen this
        let seen = elem.1.fetch_add(1, Ordering::Relaxed);

        // If we just incremented the number of actors that have seen the message past/to the number of receivers,
        // and this is the last message, then pop this message, otherwise, clone it before returning
        if seen + 1 >= self.receivers && index + 1 >= self.queue.len() {
            
            // Return None if this is None. It shouldn't be, though, because we didn't exit at the previous let Some.
            self.queue.pop_back().map(|v| v.0)
        } else {
            Some(elem.0.clone())
        }
    }
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
        assert!(!inner.push(0));
        assert_eq!(inner.queue.len(), 0);

        // Now we simulate two receivers
        let mut r1_tail = 0;
        let mut r2_tail = 0;
        inner.receivers = 2;

        // Popping with no elements should return None, and should not increment either position
        assert_eq!(inner.pop(&mut r1_tail), None);
        assert_eq!(inner.pop(&mut r2_tail), None);
        assert_eq!(r1_tail, 0);
        assert_eq!(r2_tail, 0);

        // Now, because there are receivers, pushing should return true
        assert!(inner.push(1));
        assert!(inner.push(2));
        assert_eq!(inner.queue.len(), 2);

        // Both receivers should receive the first value
        // when called in order, and increment their counters.
        assert_eq!(inner.pop(&mut r1_tail), Some(1));
        assert_eq!(r1_tail, 1);
        assert_eq!(inner.pop(&mut r2_tail), Some(1));
        assert_eq!(r2_tail, 1);
        assert_eq!(inner.queue.len(), 1);
        
        // When executed out of order, the same should work.
        assert_eq!(inner.pop(&mut r2_tail), Some(2));
        assert_eq!(r2_tail, 2);
        assert_eq!(inner.pop(&mut r1_tail), Some(2));
        assert_eq!(r1_tail, 2);
        assert_eq!(inner.queue.len(), 0);

        // It should again return None, and no incrementation should happen
        assert_eq!(inner.pop(&mut r1_tail), None);
        assert_eq!(inner.pop(&mut r2_tail), None);
        assert_eq!(r1_tail, 2);
        assert_eq!(r2_tail, 2);

        // Now push two more values
        assert!(inner.push(3));
        assert!(inner.push(4));

        // And if both are done two at a time, it should not remove either until both others are done (but should increment the counters)
        assert_eq!(inner.pop(&mut r1_tail), Some(3));
        assert_eq!(inner.pop(&mut r1_tail), Some(4));
        // Popping this one again should return None and not increment
        assert_eq!(inner.pop(&mut r1_tail), None);
        assert_eq!(r1_tail, 4);
        // And the queue should have two entries
        assert_eq!(inner.queue.len(), 2);

        // Same for the other receiver, except the queue should no longer have any entries
        assert_eq!(inner.pop(&mut r2_tail), Some(3));
        assert_eq!(inner.pop(&mut r2_tail), Some(4));
        assert_eq!(r2_tail, 4);
        assert_eq!(inner.queue.len(), 0);

        // After all of this, head should be 4
        assert_eq!(inner.head, 4);

        // Now, lets set head and both indexes to usize::MAX
        inner.head = usize::MAX;
        r1_tail = usize::MAX;
        r2_tail = usize::MAX;

        // Now, assert that reading both does nothing
        assert_eq!(inner.pop(&mut r1_tail), None);
        assert_eq!(inner.pop(&mut r2_tail), None);

        // Assert that pushing one more works
        assert!(inner.push(5));
        
        // And that it wraps to one and sets the wrapped falg
        assert_eq!(inner.head, 1);
        assert_eq!(inner.observed_wrap, 0);
        assert!(inner.wrapped);

        // Now, pop one more from one of the channels
        assert_eq!(inner.pop(&mut r1_tail), Some(5));
        // observed_wrap should have incremented, wrapped should still be true,
        // and the tail should be 1
        assert_eq!(inner.observed_wrap, 1);
        assert!(inner.wrapped);
        assert_eq!(r1_tail, 1);

        // Pop from the last channel
        assert_eq!(inner.pop(&mut r2_tail), Some(5));
        // And now, observed_wrap should have reset to zero, wrapped reset to false,
        // and tail should be 1
        assert_eq!(inner.observed_wrap, 0);
        assert!(!inner.wrapped);
        assert_eq!(r2_tail, 1);

        // And now head should be 1
        assert_eq!(inner.head, 1);
    }
}