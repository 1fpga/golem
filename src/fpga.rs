use std::ffi::{c_char, c_int};

extern "C" {
    pub fn fpga_io_init() -> c_int;

    pub fn fpga_spi(word: u16) -> u16;
    pub fn is_fpga_ready(quick: c_int) -> c_int;
    pub fn fpga_wait_to_reset();

    pub fn fpga_load_rbf(name: *const c_char, cfg: *const c_char, xml: *const c_char) -> c_int;
}
