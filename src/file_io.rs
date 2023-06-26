use std::ffi::{c_char, c_int};

extern "C" {
    pub fn FindStorage();
    pub fn getRootDir() -> *const u8;
    pub fn isXmlName(path: *const c_char) -> c_int; // 1 - MRA, 2 - MGL
}
