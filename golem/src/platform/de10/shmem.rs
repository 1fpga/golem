#![cfg(feature = "platform_de10")]
use std::ffi::c_int;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use tracing::error;

#[export_name = "shmem_map"]
#[no_mangle]
pub unsafe extern "C" fn shmem_map_c(address: u32, size: u32) -> *mut u8 {
    static mut MEM_FD: Option<c_int> = None;

    if MEM_FD.is_none() {
        // libc expects a CString, so we need to add \0 at the end.
        let fd = libc::open(
            "/dev/mem\0".as_ptr(),
            libc::O_RDWR | libc::O_SYNC | libc::O_CLOEXEC,
        );
        if fd == -1 {
            error!("Error: Unable to open /dev/mem");
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
        error!("Error: Unable to mmap ({address:X}, {size} bytes)!");
        return std::ptr::null_mut();
    }

    res as *mut u8
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
