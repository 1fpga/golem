use std::ffi::c_int;
use std::ops::{Deref, DerefMut};

static mut MEM_FD: Option<c_int> = None;

/// Map a physical memory address to a virtual address. Returns a buffer that has access
/// to the physical memory.
///
/// Safety: This function is unsafe because it can cause undefined behavior if the given
/// address is invalid or if the given size is too large. In addition, the caller must
/// ensure that the returned buffer is not used after it is unmapped. AND the caller
/// must acknowledge that they're playing with dangerous forces.
pub fn map(address: usize, size: usize) -> &'static mut [u8] {
    let ptr = unsafe { shmem_map_c(address as u32, size as u32) };
    unsafe { std::slice::from_raw_parts_mut(ptr, size) }
}

pub fn unmap(map: &'static [u8]) -> bool {
    unsafe { shmem_unmap_c(map.as_ptr(), map.len() as u32) != 0 }
}

pub struct Mapper(&'static mut [u8]);

impl Mapper {
    pub fn new(address: usize, size: usize) -> Self {
        let ptr = unsafe { shmem_map_c(address as u32, size as u32) };

        Self(unsafe { std::slice::from_raw_parts_mut(ptr, size) })
    }
}

impl Deref for Mapper {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl DerefMut for Mapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl Drop for Mapper {
    fn drop(&mut self) {
        unsafe {
            shmem_unmap_c(self.0.as_ptr(), self.0.len() as u32);
        }
    }
}

#[cfg(feature = "platform_de10")]
#[export_name = "shmem_map"]
#[no_mangle]
pub unsafe extern "C" fn shmem_map_c(address: u32, size: u32) -> *mut u8 {
    if MEM_FD.is_none() {
        // libc expects a CString, so we need to add \0 at the end.
        let fd = libc::open(
            "/dev/mem\0".as_ptr(),
            libc::O_RDWR | libc::O_SYNC | libc::O_CLOEXEC,
        );
        if fd == -1 {
            println!("Error: Unable to open /dev/mem");
            return std::ptr::null_mut();
        }
        MEM_FD = Some(fd);
    }

    let fd = MEM_FD.expect("a file descriptor");
    let res = libc::mmap(
        std::ptr::null_mut(),
        size as libc::size_t,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
        fd,
        address as libc::off_t,
    );
    if res == libc::MAP_FAILED {
        println!("Error: Unable to mmap ({address:?}, {size})!");
        return std::ptr::null_mut();
    }

    res as *mut u8
}

#[cfg(not(feature = "platform_de10"))]
pub unsafe extern "C" fn shmem_map_c(_address: u32, _size: u32) -> *mut u8 {
    std::ptr::null_mut()
}

#[export_name = "shmem_unmap"]
#[no_mangle]
pub unsafe fn shmem_unmap_c(map: *const u8, size: u32) -> c_int {
    if libc::munmap(map as *mut libc::c_void, size as libc::size_t) < 0 {
        println!("Error: Unable to unmap({map:?}, {size})!");
        return 0;
    }

    1
}

#[export_name = "shmem_put"]
#[no_mangle]
pub unsafe fn shmem_put_c(address: u32, size: u32, buf: *const u8) -> c_int {
    let shmem = shmem_map_c(address, size);
    if !shmem.is_null() {
        libc::memcpy(
            shmem as *mut libc::c_void,
            buf as *const libc::c_void,
            size as libc::size_t,
        );
        shmem_unmap_c(shmem, size);
        1
    } else {
        0
    }
}

#[export_name = "shmem_get"]
#[no_mangle]
pub unsafe fn shmem_get_c(address: u32, size: u32, buf: *const u8) -> c_int {
    let shmem = shmem_map_c(address, size);
    if !shmem.is_null() {
        libc::memcpy(
            buf as *mut libc::c_void,
            shmem as *const libc::c_void,
            size as libc::size_t,
        );
        shmem_unmap_c(shmem, size);
        1
    } else {
        0
    }
}
