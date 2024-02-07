#![cfg(feature = "std")]

use crate::memory::MemoryMapper;
use std::pin::Pin;

/// Maps a region of memory over a vector.
/// Useful for testing.
pub struct BufferMemoryMapper {
    region: Pin<Box<[u8]>>,
}

impl BufferMemoryMapper {
    pub fn new(size: usize) -> Self {
        Self {
            region: vec![0; size].into_boxed_slice().into(),
        }
    }

    pub fn from_vec(vec: Vec<u8>) -> Self {
        Self {
            region: vec.into_boxed_slice().into(),
        }
    }
}

impl MemoryMapper for BufferMemoryMapper {
    fn create(_address: usize, _size: usize) -> Result<Self, &'static str>
    where
        Self: Sized,
    {
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
