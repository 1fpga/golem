extern "C" {
    pub fn cfg_bootcore_timeout() -> u16;
    pub fn cfg_set_bootcore_timeout(timeout: u16);
    pub fn cfg_bootcore() -> *const u8;
    pub fn cfg_set_bootcore(path: *const u8);
}
