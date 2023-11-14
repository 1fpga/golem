pub mod dev_mem;

#[cfg(feature = "std")]
pub use dev_mem::*;
use std::ops::{Bound, RangeBounds};

pub mod buffer;
#[cfg(feature = "std")]
pub use buffer::*;

/// Maps a memory region to a pointer.
pub trait MemoryMapper {
    /// Create a Mapper that maps the given physical address and size.
    fn create(address: usize, size: usize) -> Result<Self, &'static str>
    where
        Self: Sized;

    /// Returns the maximum length that can be addressed. The end of this
    /// memory mapped range is `address + len()`.
    fn len(&self) -> usize;

    /// Returns true if the mapping is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a pointer to the mapped memory region.
    fn as_ptr<T>(&self) -> *const T;

    /// Returns a mutable pointer to the mapped memory region.
    fn as_mut_ptr<T>(&mut self) -> *mut T;

    /// Creates an inner range of bytes.
    fn as_range(&self, range: impl RangeBounds<usize>) -> &[u8] {
        let max = self.len();
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start + 1,
            Bound::Unbounded => 0,
        }
        .clamp(0, max);
        let end = match range.end_bound() {
            Bound::Included(end) => *end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => max,
        }
        .min(max);

        unsafe { std::slice::from_raw_parts(self.as_ptr::<u8>().add(start), end - start) }
    }

    /// Creates an inner mutable range of bytes.
    fn as_mut_range(&mut self, range: impl RangeBounds<usize>) -> &mut [u8] {
        let max = self.len();

        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start + 1,
            Bound::Unbounded => 0,
        }
        .clamp(0, max);
        let end = match range.end_bound() {
            Bound::Included(end) => *end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => max,
        }
        .min(max);

        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr::<u8>().add(start), end - start) }
    }
}

pub struct RegionMemoryMapper<'a> {
    region: &'a mut [u8],
}

impl<'a> RegionMemoryMapper<'a> {
    pub fn new(region: &'a mut [u8]) -> Self {
        Self { region }
    }
}

impl<'a> MemoryMapper for RegionMemoryMapper<'a> {
    fn create(_address: usize, _size: usize) -> Result<Self, &'static str> {
        unimplemented!()
    }

    fn len(&self) -> usize {
        self.region.len()
    }

    fn as_ptr<T>(&self) -> *const T {
        self.region.as_ptr() as *const T
    }

    fn as_mut_ptr<T>(&mut self) -> *mut T {
        self.region.as_mut_ptr() as *mut T
    }
}
