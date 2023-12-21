use std::ffi::c_int;

extern "C" {
    pub fn input_poll(getchar: c_int) -> c_int;
}
