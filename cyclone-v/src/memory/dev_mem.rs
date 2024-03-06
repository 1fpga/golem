#![cfg(feature = "std")]
use crate::memory::MemoryMapper;
use std::fmt;
use std::ops::{Index, RangeBounds};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;

/// A memory mapper that maps over the `/dev/mem` physical memory device.
/// Only available on Linux with `std` feature enabled.
pub struct DevMemMemoryMapper {
    region: &'static [u8],
    physical: (usize, usize),
}

impl fmt::Debug for DevMemMemoryMapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DevMemMemoryMapper")
            .field(
                "region",
                &format_args!("0x{:p}..0x{:p}", self.region.as_ptr(), unsafe {
                    self.region.as_ptr().add(self.region.len())
                }),
            )
            .field(
                "physical",
                &format_args!("{:#X} ({} bytes)", self.physical.0, self.physical.1),
            )
            .finish()
    }
}

impl MemoryMapper for DevMemMemoryMapper {
    fn create(address: usize, size: usize) -> Result<Self, &'static str> {
        unsafe {
            let file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(libc::O_SYNC | libc::O_CLOEXEC)
                .open("/dev/mem")
                .map_err(|_| "Unable to open /dev/mem")?;
            let fd = file.as_raw_fd();

            let res = libc::mmap(
                std::ptr::null_mut(),
                size as libc::size_t,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                address as libc::off_t,
            );

            if res == libc::MAP_FAILED {
                return Err("Unable to map memory region");
            }

            Ok(Self {
                region: core::slice::from_raw_parts_mut(res as *mut u8, size),
                physical: (address, size),
            })
        }
    }

    fn len(&self) -> usize {
        self.region.len()
    }

    fn as_ptr<T>(&self) -> *const T {
        self.region.as_ptr() as *const T
    }

    fn as_mut_ptr<T>(&mut self) -> *mut T {
        self.region.as_ptr() as *mut T
    }
}

impl Drop for DevMemMemoryMapper {
    fn drop(&mut self) {
        unsafe {
            // We unfortunately cannot care if this fails, as we are in a drop function.
            let _ = libc::munmap(self.region.as_ptr() as *mut libc::c_void, self.region.len());
        }
    }
}

impl<R: RangeBounds<usize>> Index<R> for DevMemMemoryMapper {
    type Output = [u8];

    fn index(&self, index: R) -> &Self::Output {
        self.as_range(index)
    }
}
