use std::ffi::c_int;

static mut MEM_FD: Option<c_int> = None;

#[no_mangle]
pub unsafe extern "C" fn shmem_map(address: u32, size: u32) -> *mut u8 {
    if MEM_FD == None {
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

    return res as *mut u8;
}

#[no_mangle]
pub unsafe fn shmem_unmap(map: *const u8, size: u32) -> c_int {
    if libc::munmap(map as *mut libc::c_void, size as libc::size_t) < 0 {
        println!("Error: Unable to unmap({map:?}, {size})!");
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe fn shmem_put(address: u32, size: u32, buf: *const u8) -> c_int {
    let shmem = shmem_map(address, size);
    if !shmem.is_null() {
        libc::memcpy(
            shmem as *mut libc::c_void,
            buf as *const libc::c_void,
            size as libc::size_t,
        );
        shmem_unmap(shmem, size);
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe fn shmem_get(address: u32, size: u32, buf: *const u8) -> c_int {
    let shmem = shmem_map(address, size);
    if !shmem.is_null() {
        libc::memcpy(
            buf as *mut libc::c_void,
            shmem as *const libc::c_void,
            size as libc::size_t,
        );
        shmem_unmap(shmem, size);
        1
    } else {
        0
    }
}
