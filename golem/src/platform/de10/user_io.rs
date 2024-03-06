use std::ffi::{c_char, c_int, CStr};
use tracing::debug;

#[cfg(feature = "platform_de10")]
extern "C" {
    pub fn user_io_osd_key_enable(enabled: u8);
    pub fn user_io_init(path: *const c_char, xml: *const c_char);

    pub fn is_menu() -> u8;
}

#[no_mangle]
pub extern "C" fn user_io_init_rust(path: *const c_char, xml: *const c_char) -> u8 {
    let path = if path.is_null() {
        ""
    } else {
        unsafe { CStr::from_ptr(path) }.to_str().unwrap()
    };
    let xml = if xml.is_null() {
        ""
    } else {
        unsafe { CStr::from_ptr(xml) }.to_str().unwrap()
    };

    debug!(path, xml);

    return 0;
}

#[no_mangle]
pub extern "C" fn altcfg(alt: c_int) -> u16 {
    0
}
