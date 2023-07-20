use std::ffi::c_int;

#[cfg(feature = "de10")]
extern "C" {
    pub fn input_poll(getchar: c_int) -> c_int;
}

#[cfg(not(feature = "de10"))]
pub fn input_poll(getchar: c_int) -> c_int {
    0
}
