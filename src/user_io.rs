#![cfg(feature = "platform_de10")]
use crate::shmem;
use std::ffi::{c_char, c_int, CStr};
use tracing::debug;

#[cfg(feature = "platform_de10")]
#[allow(unused)]
extern "C" {
    pub fn user_io_osd_key_enable(enabled: u8);
    pub fn user_io_poll();
    pub fn user_io_init(path: *const c_char, xml: *const c_char);

    pub fn is_menu() -> u8;
    pub fn is_x86() -> u8;
    pub fn is_snes() -> u8;
    pub fn is_sgb() -> u8;
    pub fn is_neogeo() -> u8;
    pub fn is_neogeo_cd() -> u8;
    pub fn is_megacd() -> u8;
    pub fn is_pce() -> u8;
    pub fn is_archie() -> u8;
    pub fn is_gba() -> u8;
    pub fn is_c64() -> u8;
    pub fn is_st() -> u8;
    pub fn is_psx() -> u8;
    pub fn is_arcade() -> u8;
    pub fn is_saturn() -> u8;
    pub fn is_pcxt() -> u8;
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

    return 1;
}

#[no_mangle]
pub extern "C" fn altcfg(alt: c_int) -> u16 {
    let mut map = shmem::Mapper::new(0x1FFFF000, 0x1000);
    let par = &mut map[0xF04..];
    if alt >= 0 {
        par[0] = 0x34;
        par[1] = 0x99;
        par[2] = 0xBA;
        par[3] = alt as u8;

        println!("** altcfg({alt})");
    } else if (par[0] == 0x34) && (par[1] == 0x99) && (par[2] == 0xBA) {
        let res = par[3];
        println!("** altcfg: got config {res}");
        return res as u16;
    } else {
        println!("** altcfg: no config");
    }

    0
}
