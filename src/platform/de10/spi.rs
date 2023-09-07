#![cfg(feature = "platform_de10")]

use std::ffi::c_int;

const SSPI_IO_EN: u32 = 1u32 << 20;
pub const UIO_GET_VRES: u16 = 0x23;
pub const UIO_GET_FB_PAR: u16 = 0x40;

extern "C" {
    pub fn spi_osd_cmd(cmd: u8);
    pub fn spi_osd_cmd_cont(cmd: u8);
    pub fn spi_uio_cmd_cont(cmd: u16) -> u16;
    pub fn spi_write(addr: *const u8, len: u32, wide: c_int);
    // pub fn spi_w(data: u16) -> u16;
    // pub fn DisableIo();
    pub fn fpga_spi(word: u16) -> u16;
    pub fn fpga_spi_en(mask: u32, en: u32);

    pub fn DisableOsd();
}

#[inline]
pub fn spi_w(word: u16) -> u16 {
    unsafe { fpga_spi(word) }
}

#[no_mangle]
pub unsafe extern "C" fn DisableIo() {
    fpga_spi_en(SSPI_IO_EN, 0);
}
