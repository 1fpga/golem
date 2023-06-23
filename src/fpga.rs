use std::ffi::c_int;

extern "C" {
    pub fn fpga_spi(word: u16) -> u16;
    pub fn is_fpga_ready(quick: c_int) -> c_int;
    pub fn fpga_wait_to_reset();
}
