use std::ffi::{c_char, c_int, CString, OsStr};
use std::path::PathBuf;

#[cfg(feature = "platform_de10")]
extern "C" {
    pub fn FindStorage();
    pub fn getRootDir() -> *const u8;
    pub fn isXmlName(path: *const c_char) -> c_int; // 1 - MRA, 2 - MGL
}

#[cfg(not(feature = "platform_de10"))]
pub fn FindStorage() {}

#[cfg(not(feature = "platform_de10"))]
pub fn getRootDir() -> *const u8 {
    static mut ROOT_DIR: Option<CString> = None;

    unsafe {
        if ROOT_DIR.is_none() {
            ROOT_DIR =
                Some(CString::new(std::env::current_dir().unwrap().to_str().unwrap()).unwrap());
        }

        ROOT_DIR.as_ref().unwrap().as_ptr() as *const u8
    }
}

#[cfg(not(feature = "platform_de10"))]
pub fn isXmlName(_path: *const c_char) -> c_int {
    0
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
