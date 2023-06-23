use std::ffi::c_ulong;

extern "C" {
    pub fn GetTimer(offset: c_ulong) -> c_ulong;
    pub fn CheckTimer(t: c_ulong) -> c_ulong;
}
