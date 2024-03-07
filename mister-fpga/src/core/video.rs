use crate::config;
use tracing::{error, warn};

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
use linux as private;

#[cfg(not(target_os = "linux"))]
mod private {
    use crate::config;
    use tracing::debug;

    pub fn hdmi_config_init(config: &config::MisterConfig) -> Result<(), String> {
        debug!(?config, "HDMI configuration not supported on this platform");
        Ok(())
    }

    pub fn init_mode(
        options: &config::MisterConfig,
        _core: &mut crate::core::MisterFpgaCore,
        _is_menu: bool,
    ) -> Result<(), String> {
        debug!(
            ?options,
            "Video mode configuration not supported on this platform"
        );
        Ok(())
    }
}

/// Initialize the video Hardware configuration.
// TODO: this should not take the whole config but a subset of it related only to video.
pub fn init(options: &config::MisterConfig) {
    if let Err(error) = private::hdmi_config_init(options) {
        error!("Failed to initialize HDMI configuration: {}", error);
        warn!("This is not a fatal error, the application will continue to run.");
    }
}

pub fn init_mode(
    options: &config::MisterConfig,
    core: &mut crate::core::MisterFpgaCore,
    is_menu: bool,
) {
    if !is_menu {
        return;
    }
    if let Err(error) = private::init_mode(options, core, is_menu) {
        error!("Failed to initialize video mode: {}", error);
        warn!("This is not a fatal error, the application will continue to run.");
    }
}

/*
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

pub const UIO_GET_VRES: u16 = 0x23;
pub const UIO_GET_FB_PAR: u16 = 0x40;

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
            self.resolution = Resolution::new(read_u32() as u16, read_u32() as u16);
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
}
*/
