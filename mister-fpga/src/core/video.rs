use crate::config;
use cyclone_v::memory::MemoryMapper;
use std::time::Duration;
use tracing::{error, warn};

#[cfg(target_os = "linux")]
mod linux;

use crate::config::aspect::AspectRatio;
use crate::config::resolution::Resolution;
use crate::fpga::user_io::UserIoCommands;
use crate::fpga::Spi;
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

    arx: u16,
    ary: u16,
    arxy: u16,
    fb_en: u16,
    fb_fmt: u16,
    fb_resolution: u16,
    fb_width: u16,
    fb_height: u16,

    interlaced: bool,
    rotated: bool,

    res: u16,
    fb_crc: u16,
}

pub const UIO_GET_VRES: u16 = 0x23;
pub const UIO_GET_FB_PAR: u16 = 0x40;

impl VideoInfo {
    /// Create a video info from the FPGA.
    pub(crate) fn create(spi: &mut Spi<impl MemoryMapper>) -> Result<Self, String> {
        let mut result = VideoInfo::default();

        {
            let mut command = spi.command(UserIoCommands::UserIoGetVres);
            let mut new_res = 0;
            command.write_read(0, &mut new_res);
            let mut read_u32 = || {
                let mut high: u16 = 0;
                let mut low: u16 = 0;
                command.write_read(0, &mut low).write_read(0, &mut high);
                (high as u32) << 16 | (low as u32)
            };

            result.res = new_res;
            result.resolution = Resolution::new(read_u32() as u16, read_u32() as u16);
            result.htime_ms = read_u32();
            result.vtime_ms = read_u32();
            result.ptime_ms = read_u32();
            result.vtimeh = read_u32();
            result.ctime_ms = read_u32();
            result.interlaced = (new_res & 0x100) != 0;
            result.rotated = (new_res & 0x200) != 0;
        }

        {
            let mut crc: u16 = 0;
            let mut command = spi.command_read(UserIoCommands::UserIoGetFbParams, &mut crc);

            result.fb_crc = crc;
            command.write_read(0, &mut result.arx);
            result.arxy = !!(result.arx & 0x1000);
            result.arx &= 0xFFF;
            command.write_read(0, &mut result.ary);
            result.ary &= 0xFFF;

            command.write_read(0, &mut result.fb_fmt);
            command.write_read(0, &mut result.fb_width);
            command.write_read(0, &mut result.fb_height);
            result.fb_en = !!(result.fb_fmt & 0x40);
        }
        Ok(result)
    }

    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    pub fn aspect_ratio(&self) -> AspectRatio {
        self.aspect_ratio
    }

    pub fn vtime(&self) -> Duration {
        Duration::from_nanos(self.vtime_ms as u64 * 10)
    }
}
