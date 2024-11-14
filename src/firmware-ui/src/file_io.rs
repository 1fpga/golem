#![cfg(feature = "platform_de10")]
use std::ffi::{c_char, OsStr};
use std::path::PathBuf;

#[cfg(feature = "platform_de10")]
fn get_root_dir() -> *const u8 {
    b"/media/fat\0".as_ptr()
}

#[cfg(not(feature = "platform_de10"))]
pub fn get_root_dir() -> *const u8 {
    static mut ROOT_DIR: Option<std::ffi::CString> = None;

    unsafe {
        if ROOT_DIR.is_none() {
            ROOT_DIR = Some(
                std::ffi::CString::new(std::env::current_dir().unwrap().to_str().unwrap()).unwrap(),
            );
        }

        ROOT_DIR.as_ref().unwrap().as_ptr() as *const u8
    }
}

pub fn root_dir() -> PathBuf {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    unsafe {
        use std::os::unix::ffi::OsStrExt;

        let root_dir = std::ffi::CStr::from_ptr(get_root_dir() as *const c_char);
        PathBuf::from(OsStr::from_bytes(root_dir.to_bytes()))
    }

    // Unoptimized version for other OSes.
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    unsafe {
        let root_dir = std::ffi::CStr::from_ptr(get_root_dir() as *const c_char);
        PathBuf::from(root_dir.to_str().unwrap())
    }
}
