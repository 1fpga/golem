pub mod dev_mem;

#[cfg(feature = "std")]
pub use dev_mem::*;
use std::ops::{Bound, RangeBounds};

pub mod buffer;
#[cfg(feature = "std")]
pub use buffer::*;

fn clamp_range(range: impl RangeBounds<usize>, max: usize) -> (usize, usize) {
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

    (start, end - start)
}

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

    /// Creates an inner range of bytes. The offsets are relative to the base
    /// of the mapped memory, e.g. `as_range(0..4)` will return the first 4
    /// bytes of the mapped memory (a memory mapping to address 0x12340000 will
    /// map 0x12340000..0x12340004).
    fn as_range(&self, range: impl RangeBounds<usize>) -> &[u8] {
        let (start, len) = clamp_range(range, self.len());
        unsafe { std::slice::from_raw_parts(self.as_ptr::<u8>().add(start), len) }
    }

    /// Creates an inner mutable range of bytes.
    fn as_mut_range(&mut self, range: impl RangeBounds<usize>) -> &mut [u8] {
        let (start, len) = clamp_range(range, self.len());
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr::<u8>().add(start), len) }
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

#[test]
fn range_works() {
    let data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut data2 = data;
    let mut mapper = RegionMemoryMapper::new(&mut data2);
    assert_eq!(mapper.as_range(..), &data);
    assert_eq!(mapper.as_range(0..99), &data);
    assert_eq!(mapper.as_range(5..8), &data[5..8]);

    mapper.as_mut_range(5..8).copy_from_slice(&[0, 0, 0]);
    assert_eq!(mapper.as_range(..), &[0, 1, 2, 3, 4, 0, 0, 0, 8, 9]);
}
