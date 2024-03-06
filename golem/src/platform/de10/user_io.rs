use std::ffi::{c_char, c_int, CStr};

#[cfg(feature = "platform_de10")]
extern "C" {
    pub fn user_io_init(path: *const c_char, xml: *const c_char);
}
