use std::ffi::c_char;

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
