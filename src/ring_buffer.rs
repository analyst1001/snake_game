/* Generic RingBuffer implementation
//
// Not tested yet, may contain bugs
// TODO: Write unit tests for RingBuffer
*/

use crate::{print, println};
use core::iter::Iterator;

/// Thread unsafe RingBuffer over an array
#[derive(Debug)]
pub struct RingBuffer<'a, T> {
    /// Statically allocated buffer, since we don't have dynamic memory allocation yet :(
    buffer: &'a mut [T],
    /// Invariant: The valid elements in buffer are in range [first, last)
    first: usize,
    last: usize,
    /// True when buffer is full. False otherwise.
    full: bool,
}

impl<'a, T> RingBuffer<'a, T> {
    /// Create a new ring buffer utilizing provided array as buffer space
    pub fn new(buffer: &'a mut [T]) -> Self {
        RingBuffer {
            buffer: buffer,
            first: 0,
            last: 0,
            full: false,
        }
    }

    /// Check if ring buffer is completely empty
    fn is_empty(&self) -> bool {
        self.first == self.last && self.full == false
    }

    /// Check if ring buffer is completely filled up
    fn is_full(&self) -> bool {
        self.first == self.last && self.full == true
    }

    /// Put an element at the beginning of the buffer, logically shifting existing elements to the right in the process
    pub fn prepend(&mut self, element: T) {
        assert!(!self.is_full());
        // May have overflow
        let new_first_pos = (self.first + self.buffer.len() - 1) % self.buffer.len();
        self.buffer[new_first_pos] = element;
        self.first = new_first_pos;
        if new_first_pos == self.last {
            self.full = true;
        }
    }

    /// Put an element at the end of the buffer
    pub fn append(&mut self, element: T) {
        assert!(!self.is_full());
        let new_last_pos = (self.last + 1) % self.buffer.len();
        self.buffer[self.last] = element;
        self.last = new_last_pos;
        if new_last_pos == self.first {
            self.full = true;
        }
    }

    /// Take a peek at the first element in the buffer
    pub fn peek_first(&self) -> &T {
        assert!(!self.is_empty());
        &self.buffer[self.first]
    }

    /// Take a peek at the last element in the buffer
    pub fn peek_last(&self) -> &T {
        assert!(!self.is_empty());
        // May have overflow
        &self.buffer[(self.last + self.buffer.len() - 1) % self.buffer.len()]
    }

    /// Take a look at the i'th element in the buffer
    pub fn peek_ith(&self, i: usize) -> &T {
        assert!(!self.is_empty());
        assert!(i < self.buffer.len());
        let index = (self.first + i) % self.buffer.len();
        //println!("index {}, self.last {}",index, self.last);
        //assert!(index < self.last);
        &self.buffer[index]
    }

    /// Remove and return the first element from the buffer, logically shifting other existing elements to left
    pub fn pop_first(&mut self) -> &T {
        assert!(!self.is_empty());
        let element = &self.buffer[self.first];
        self.first = (self.first + 1) % self.buffer.len();
        self.full = false;
        element
    }

    /// Remove and return the last element from the buffer
    pub fn pop_last(&mut self) -> &T {
        assert!(!self.is_empty());
        // May have overflow
        self.last = (self.last + self.buffer.len() - 1) % self.buffer.len();
        self.full = false;
        &self.buffer[self.last]
    }

    /// Return an iterator over triplets of values inside the ring buffer
    // Assumes minimum size of 3
    pub fn triple_iter<'b>(&'b self) -> RingBufferTripletsIterator<'b, T> {
        RingBufferTripletsIterator {
            ring_buffer: self,
            current_index: self.first,
        }
    }
}

/// Iterator over triplets of values inside the ring buffer
pub struct RingBufferTripletsIterator<'a, T> {
    ring_buffer: &'a RingBuffer<'a, T>,
    current_index: usize,
}

impl<'a, T> Iterator for RingBufferTripletsIterator<'a, T> {
    type Item = (&'a T, &'a T, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        if (self.current_index + 2) % self.ring_buffer.buffer.len() == self.ring_buffer.last {
            return None;
        }
        let current_index = self.current_index - self.ring_buffer.first;
        //println!("current_index {}, first: {}", self.current_index, self.ring_buffer.first);
        let (first_element, second_element, third_element) = (
            self.ring_buffer.peek_ith(current_index),
            self.ring_buffer.peek_ith(current_index + 1),
            self.ring_buffer.peek_ith(current_index + 2),
        );
        self.current_index += 1;
        return Some((first_element, second_element, third_element));
    }
}
