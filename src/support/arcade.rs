use std::ffi::{c_char, c_int};

extern "C" {
    pub fn xml_load(xml: *const c_char) -> c_int;
}
