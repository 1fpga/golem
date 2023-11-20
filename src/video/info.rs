#![allow(dead_code)]
use crate::video::aspect::AspectRatio;
use crate::video::resolution::Resolution;

mod ffi {
    /// C++ structure for the video info, should be byte-compatible with the C version in `video.h`.
    #[repr(C)]
    pub struct VideoInfo {
        pub width: u32,
        pub height: u32,
        pub htime: u32,
        pub vtime: u32,
        pub ptime: u32,
        pub ctime: u32,
        pub vtimeh: u32,
        pub arx: u32,
        pub ary: u32,
        pub arxy: u32,
        pub fb_en: u32,
        pub fb_fmt: u32,
        pub fb_width: u32,
        pub fb_height: u32,

        pub interlaced: bool,
        pub rotated: bool,
    }
}

#[derive(Debug, Default)]
pub struct VideoInfo {
    resolution: Resolution,
    aspect_ratio: AspectRatio,

    // TODO: figure out what these timing numbers mean.
    htime_ms: u32,
    vtime_ms: u32,
    ptime_ms: u32,
    ctime_ms: u32,
    vtimeh: u32,

    arx: u32,
    ary: u32,
    arxy: u32,
    fb_en: u32,
    fb_fmt: u32,
    fb_resolution: u32,
    fb_width: u32,
    fb_height: u32,

    interlaced: bool,
    rotated: bool,

    res: u16,
    fb_crc: u16,
}

impl VideoInfo {
    /// Update the video info from the FPGA. Returns true if the video info changed.
    #[cfg(all(not(test), feature = "platform_de10"))]
    pub fn update(&mut self, force: bool) -> bool {
        use crate::platform::de10::spi;

        fn read_u32() -> u32 {
            spi::spi_w(0) as u32 | ((spi::spi_w(0) as u32) << 16)
        }
        fn read_u16() -> u32 {
            spi::spi_w(0) as u32
        }

        unsafe {
            spi::spi_uio_cmd_cont(spi::UIO_GET_VRES);
        }

        let new_res = spi::spi_w(0);
        let res_changed = new_res != self.res || force;
        if new_res != self.res {
            self.res = new_res;
            self.resolution = Resolution::new(read_u32(), read_u32());
            self.htime_ms = read_u32();
            self.vtime_ms = read_u32();
            self.ptime_ms = read_u32();
            self.vtimeh = read_u32();
            self.ctime_ms = read_u32();
            self.interlaced = (new_res & 0x100) != 0;
            self.rotated = (new_res & 0x200) != 0;
        }

        unsafe {
            crate::platform::de10::fpga::ffi::DisableIO();
        }

        let crc = unsafe { spi::spi_uio_cmd_cont(spi::UIO_GET_FB_PAR) };
        let fb_changed = self.fb_crc != crc;
        if fb_changed || res_changed {
            self.fb_crc = crc;
            self.arx = read_u16();
            self.arxy = !!(self.arx & 0x1000);
            self.arx &= 0xFFF;
            self.ary = read_u16() & 0xFFF;
            self.fb_fmt = read_u16();
            self.fb_width = read_u16();
            self.fb_height = read_u16();
            self.fb_en = !!(self.fb_fmt & 0x40);
        }

        res_changed || fb_changed
    }

    /// Update the C++ video info struct from this internal Rust representation.
    #[cfg(not(test))]
    pub unsafe fn set_cpp_info(&self, info: *mut ffi::VideoInfo) {
        let info = &mut *info;

        info.width = self.resolution.width;
        info.height = self.resolution.height;
        info.htime = self.htime_ms;
        info.vtime = self.vtime_ms;
        info.ptime = self.ptime_ms;
        info.ctime = self.ctime_ms;
        info.vtimeh = self.vtimeh;
        info.arx = self.arx;
        info.ary = self.ary;
        info.arxy = self.arxy;
        info.fb_en = self.fb_en;
        info.fb_fmt = self.fb_fmt;
        info.fb_width = self.fb_width;
        info.fb_height = self.fb_height;
        info.interlaced = self.interlaced;
        info.rotated = self.rotated;
    }
}

#[cfg(all(not(test), feature = "platform_de10"))]
#[no_mangle]
unsafe extern "C" fn get_video_info_rust(force: u8, info: *mut ffi::VideoInfo) -> u8 {
    static mut GLOBAL_INFO: std::mem::MaybeUninit<VideoInfo> = std::mem::MaybeUninit::uninit();

    let i = GLOBAL_INFO.assume_init_mut();

    if i.update(force != 0) {
        i.set_cpp_info(info);
        1
    } else {
        0
    }
}
