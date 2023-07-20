use crate::shmem;
use std::ffi::{c_char, c_int};

#[cfg(feature = "de10")]
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

#[cfg(not(feature = "de10"))]
mod de10_impl {
    use super::*;

    pub fn user_io_osd_key_enable(enabled: u8) {}
    pub fn user_io_poll() {}
    pub fn user_io_init(path: *const c_char, xml: *const c_char) {}

    pub fn is_menu() -> u8 {
        0
    }
    pub fn is_x86() -> u8 {
        0
    }
    pub fn is_snes() -> u8 {
        0
    }
    pub fn is_sgb() -> u8 {
        0
    }
    pub fn is_neogeo() -> u8 {
        0
    }
    pub fn is_neogeo_cd() -> u8 {
        0
    }
    pub fn is_megacd() -> u8 {
        0
    }
    pub fn is_pce() -> u8 {
        0
    }
    pub fn is_archie() -> u8 {
        0
    }
    pub fn is_gba() -> u8 {
        0
    }
    pub fn is_c64() -> u8 {
        0
    }
    pub fn is_st() -> u8 {
        0
    }
    pub fn is_psx() -> u8 {
        0
    }
    pub fn is_arcade() -> u8 {
        0
    }
    pub fn is_saturn() -> u8 {
        0
    }
    pub fn is_pcxt() -> u8 {
        0
    }
}

#[cfg(not(feature = "de10"))]
pub use de10_impl::*;

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
        println!("** altcfg: got cfg {res}");
        return res as u16;
    } else {
        println!("** altcfg: no cfg");
    }

    0
}
