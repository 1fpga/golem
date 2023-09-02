mod dev_mem;
#[cfg(feature = "std")]
pub use dev_mem::*;

mod buffer;
#[cfg(feature = "std")]
pub use buffer::*;

/// Maps a memory region to a pointer.
pub trait MemoryMapper {
    /// Create a Mapper that maps the given physical address and size.
    fn create(address: usize, size: usize) -> Result<Self, &'static str>
    where
        Self: Sized;

    /// Returns a pointer to the mapped memory region.
    fn as_ptr<T>(&self) -> *const T;

    /// Returns a mutable pointer to the mapped memory region.
    fn as_mut_ptr<T>(&mut self) -> *mut T;
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

    fn as_ptr<T>(&self) -> *const T {
        self.region.as_ptr() as *const T
    }

    fn as_mut_ptr<T>(&mut self) -> *mut T {
        self.region.as_mut_ptr() as *mut T
    }
}
