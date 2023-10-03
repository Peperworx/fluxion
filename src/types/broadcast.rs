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

    /// Pushes a message to the queue
    pub fn push(&mut self, message: T) {
        
        // If this would cause head to wrap, then set the wrap flag to true and reset head to zero
        if self.head == usize::MAX {
            self.head = 0;
            self.wrapped = true;
            self.observed_wrap = 0;
        } else {
            // Otherwise, just increment head
            self.head += 1;
        }

        // Push the value to the front of the queue
        self.queue.push_front((message, AtomicUsize::new(0)));
    }

    /// Pop a message from the queue at the given position
    pub fn pop(&mut self, tail: &mut usize) -> Option<T> {
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
        // The only reason why tail would be larger than head was if we wrapped. And we sure didn't wrap by popping.
        let index = head - *tail;

        // Get the element
        let elem = self.queue.get(index);

        // If we got None, return None
        let Some(elem) = elem else {
            return None;
        };

        // Otherwise, fetch and increment the number of recievers that have seen this
        let seen = elem.1.fetch_add(1, Ordering::Relaxed);

        // If we just incremented seen past the number of receivers, and we mess
        if seen +=
        
    }
}

