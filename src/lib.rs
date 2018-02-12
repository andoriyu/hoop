use std::iter::{DoubleEndedIterator, Iterator};

/// Yet another ring buffer implmentation. This one has ability to iterate both ways without
/// mutation buffer.
///
/// # Usage
///
/// ```
/// use hoop::Hoop;
///
/// let mut buffer = Hoop::with_capacity(4);
/// buffer.write('1');
/// buffer.write('2');
/// buffer.write('3');
/// buffer.write('4');
/// let mut iter = buffer.iter();
/// assert_eq!(Some(&'1'), iter.next());
/// assert_eq!(Some(&'4'), iter.next_back());
/// assert_eq!(Some(&'2'), iter.next());
/// assert_eq!(Some(&'3'), iter.next_back());
/// assert_eq!(None, iter.next());
/// assert_eq!(None, iter.next_back());
/// ```
pub struct Hoop<T: Clone> {
    inner: Vec<Option<T>>,
    // Next read
    read_position: usize,
    // Next Write
    write_position: usize,
}

impl<T: Clone> Hoop<T> {
    /// Create new ring buffer with desired capacity.
    pub fn with_capacity(capacity: usize) -> Hoop<T> {
        Hoop {
            inner: vec![None; capacity],
            read_position: 0,
            write_position: 0,
        }
    }

    /// Capacity of inner Vec.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Pop oldest item from a buffer.
    pub fn pop(&mut self) -> Option<T> {
        let ret: Option<T> = self.inner[self.read_position].take();
        if ret.is_some() {
            self.read_position = self.advance(self.read_position);
        }
        ret
    }

    /// Try writting to a buffer.
    pub fn write(&mut self, item: T) -> WriteResult {
        let idx = self.write_position;
        {
            let stored = &self.inner[idx];
            if stored.is_some() {
                return WriteResult::TooMany;
            }
        }
        self.inner[idx] = Some(item);
        self.write_position = self.advance(self.write_position);
        WriteResult::Done
    }

    /// Write even if at a capacity. This ither is a normal write or overwrite + move read position
    /// forward.
    pub fn overwrite(&mut self, item: T) {
        let idx = self.write_position;
        {
            let stored = &self.inner[idx];
            if stored.is_some() {
                self.read_position = self.advance(self.read_position);
            }
        }
        self.inner[idx] = Some(item);
        self.write_position = self.advance(self.write_position);
    }

    /// Clear buffer. This is `O(n)` operation.
    pub fn clear(&mut self) {
        self.read_position = 0;
        self.write_position = 0;
		for el in self.inner.iter_mut() {
			*el = None;
		}
    }

    /// Create non-consuming iterator.
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }

    fn advance(&self, current: usize) -> usize {
       if (current + 1) == self.capacity() {
            0
        } else {
            current + 1
        }
    }

    fn retreat(&self, current: usize) -> usize {
        if current == 0 {
            self.capacity() - 1
        } else {
            current - 1
        }
    }
}

pub struct Iter<'data, T: 'data + Clone> {
    hoop: &'data Hoop<T>,
    forward_position: usize,
    seeking_forward: bool,
    backward_position: usize,
    seeking_backward: bool,
}

impl<'data, T: 'data + Clone> Iterator for Iter<'data, T> {
    type Item = &'data T;
    fn next(&mut self) -> Option<&'data T> {
        // We looped back to the start.
        if self.seeking_forward && self.forward_position == self.hoop.read_position {
            return None;
        }
        // We reached backward_position. We allowed to look what's underneather it.
        if self.seeking_forward && self.forward_position > self.backward_position {
            return None;
        }
        if let Some(ref item) = self.hoop.inner[self.forward_position] {
            self.forward_position = self.hoop.advance(self.forward_position);
            self.seeking_forward = true;
            Some(item)
        } else {
            None
        }
    }
}

impl <'data, T: 'data + Clone> DoubleEndedIterator for Iter<'data, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        // We looped back to the start.
        if self.seeking_backward && self.backward_position == self.hoop.write_position {
            return None;
        }
        let ahead_of_reader = self.backward_position > self.hoop.read_position;
        if self.seeking_backward && ahead_of_reader && self.backward_position < self.forward_position {
            return None;
        }

        if let Some(ref item) = self.hoop.inner[self.backward_position] {
            self.backward_position = self.hoop.retreat(self.backward_position);
            self.seeking_backward = true;
            Some(item)
        } else {
            None
        }
    }
}

impl<'data, T: 'data + Clone> Iter<'data, T> {
    fn new(hoop: &'data Hoop<T>) -> Self {
        Iter {
            hoop: hoop,
            forward_position: hoop.read_position,
            backward_position: hoop.retreat(hoop.write_position),
            seeking_forward: false,
            seeking_backward: false,
        }
    }
}


#[must_use]
/// Result of a write operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WriteResult {
    /// Item was written to a buffer.
    Done,
    /// Buffer can't take any more items.
    TooMany,
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;

    #[test]
    fn error_on_read_empty_buffer() {
        let mut buffer = Hoop::<char>::with_capacity(1);
        assert_eq!(None, buffer.pop());
    }

    #[test]
    fn write_and_read_back_item() {
        let mut buffer = Hoop::with_capacity(1);
        buffer.write('1');
        assert_eq!(Some('1'), buffer.pop());
        assert_eq!(None, buffer.pop());
    }

    #[test]
    fn write_and_read_back_multiple_items() {
        let mut buffer = Hoop::with_capacity(2);
        buffer.write('1');
        buffer.write('2');
        assert_eq!(Some('1'), buffer.pop());
        assert_eq!(Some('2'), buffer.pop());
        assert_eq!(None, buffer.pop());
    }

    #[test]
    fn alternate_write_and_read() {
        let mut buffer = Hoop::with_capacity(2);
        buffer.write('1');
        assert_eq!(Some('1'), buffer.pop());
        buffer.write('2');
        assert_eq!(Some('2'), buffer.pop());
    }

    #[test]
    fn clear_buffer() {
        let mut buffer = Hoop::with_capacity(3);
        buffer.write('1');
        buffer.write('2');
        buffer.write('3');
        buffer.clear();
        assert_eq!(None, buffer.pop());
        buffer.write('1');
        buffer.write('2');
        assert_eq!(Some('1'), buffer.pop());
        buffer.write('3');
        assert_eq!(Some('2'), buffer.pop());
    }

    #[test]
    fn full_buffer_error() {
        let mut buffer = Hoop::with_capacity(2);
        buffer.write('1');
        buffer.write('2');
        assert_eq!(WriteResult::TooMany, buffer.write('3'));
    }

    #[test]
    fn overwrite_item_in_non_full_buffer() {
        let mut buffer = Hoop::with_capacity(2);
        buffer.write('1');
        buffer.overwrite('2');
        assert_eq!(Some('1'), buffer.pop());
        assert_eq!(Some('2'), buffer.pop());
        assert_eq!(None, buffer.pop());
    }

    #[test]
    fn overwrite_item_in_full_buffer() {
        let mut buffer = Hoop::with_capacity(2);
        buffer.write('1');
        buffer.write('2');
        buffer.overwrite('A');
        assert_eq!(Some('2'), buffer.pop());
        assert_eq!(Some('A'), buffer.pop());
    }

    #[test]
    fn iterator_sequence() {
        let mut buffer = Hoop::with_capacity(2);
        buffer.write('1');
        buffer.write('2');

        let expected = vec!['1', '2'];

        let result: Vec<char> = buffer.iter().cloned().collect();
        assert_eq!(expected, result);
    }

    #[test]
    fn iterator_warped() {
        let mut buffer = Hoop::with_capacity(2);
        buffer.write('1');
        buffer.write('2');
        buffer.overwrite('A');

        let expected = vec!['2', 'A'];

        let result: Vec<char> = buffer.iter().cloned().collect();
        assert_eq!(expected, result);
    }

    // Should Fail to compile
    /*
    #[test]
    fn iterator_read_and_iter() {
        let mut buffer = Hoop::with_capacity(2);
        buffer.write('1');
        buffer.write('2');

        let mut one = buffer.iter().take(1);

        let left = one.next().map(|e| e.clone());
        let right = buffer.pop();
        assert_eq!(left, right);
    }*/

    #[test]
    fn iterator_should_not_consume() {
        let mut buffer = Hoop::with_capacity(2);
        buffer.write('1');
        buffer.write('2');


        let left: Vec<&char> = buffer.iter().collect();
        let right: Vec<&char> = buffer.iter().collect();
        assert_eq!(left, right);
    }

    #[test]
    fn that_scene_from_requiem_for_dream() {
        let mut buffer = Hoop::with_capacity(4);
        buffer.write('1');
        buffer.write('2');
        buffer.write('3');
        buffer.write('4');

        let mut iter = buffer.iter();
        assert_eq!(Some(&'1'), iter.next());
        assert_eq!(Some(&'4'), iter.next_back());
        assert_eq!(Some(&'2'), iter.next());
        assert_eq!(Some(&'3'), iter.next_back());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next_back());
    }
}
