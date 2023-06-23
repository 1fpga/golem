use crate::fpga;
use std::ffi::c_int;

extern "C" {
    pub fn spi_osd_cmd(cmd: u8) -> ();
    pub fn spi_osd_cmd_cont(cmd: u8) -> ();
    pub fn spi_write(addr: *const u8, len: u32, wide: c_int);

    pub fn DisableOsd() -> ();
}

#[inline]
pub fn spi_w(word: u16) -> u16 {
    unsafe { fpga::fpga_spi(word) }
}
