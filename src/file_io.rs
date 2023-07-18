use std::ffi::{c_char, c_int, OsStr};
use std::path::PathBuf;

extern "C" {
    pub fn FindStorage();
    pub fn getRootDir() -> *const u8;
    pub fn isXmlName(path: *const c_char) -> c_int; // 1 - MRA, 2 - MGL
}

pub fn root_dir() -> PathBuf {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "unix"))]
    unsafe {
        use std::os::unix::ffi::OsStrExt;

        let root_dir = std::ffi::CStr::from_ptr(getRootDir() as *const c_char);
        PathBuf::from(OsStr::from_bytes(root_dir.to_bytes()))
    }

    // Unoptimized version for other OSes.
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "unix")))]
    unsafe {
        let root_dir = std::ffi::CStr::from_ptr(getRootDir() as *const c_char);
        PathBuf::from(root_dir.to_str().unwrap())
    }
}
