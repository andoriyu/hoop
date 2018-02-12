/// Yet another ring buffer implmentation. This one has ability to iterate both ways without
/// mutation buffer.
///
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

    fn advance(&self, current: usize) -> usize {
       if (current + 1) == self.capacity() {
            0
        } else {
            current + 1
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
}
