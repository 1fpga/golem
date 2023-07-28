use std::ffi::c_int;

#[cfg(feature = "platform_de10")]
extern "C" {
    pub fn input_poll(getchar: c_int) -> c_int;
}

#[cfg(not(feature = "platform_de10"))]
pub fn input_poll(_getchar: c_int) -> c_int {
    0
}
