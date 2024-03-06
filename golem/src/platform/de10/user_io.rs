use std::ffi::{c_char, c_int, CStr};
use tracing::debug;

#[cfg(feature = "platform_de10")]
extern "C" {
    pub fn user_io_init(path: *const c_char, xml: *const c_char);
}

#[no_mangle]
pub extern "C" fn altcfg(_alt: c_int) -> u16 {
    0
}
