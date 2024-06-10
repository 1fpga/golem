use std::time::Duration;

use tracing::{error, warn};

use cyclone_v::memory::MemoryMapper;
#[cfg(target_os = "linux")]
use linux as private;

use crate::config;
use crate::config::aspect::AspectRatio;
use crate::config::edid::CustomVideoMode;
use crate::config::resolution::Resolution;
use crate::fpga::user_io::UserIoCommands;
use crate::fpga::Spi;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(target_os = "linux"))]
mod private {
    use tracing::debug;

    use cyclone_v::memory::MemoryMapper;

    use crate::config;
    use crate::config::aspect::AspectRatio;
    use crate::config::edid::CustomVideoMode;
    use crate::fpga::Spi;

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

    pub fn select_mode(
        _mode: CustomVideoMode,
        _direct_video: bool,
        _aspect_ratio_1: Option<AspectRatio>,
        _aspect_ratio_2: Option<AspectRatio>,
        _spi: &mut Spi<impl MemoryMapper>,
        _is_menu: bool,
    ) -> Result<(), String> {
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

pub fn select_mode(
    mode: CustomVideoMode,
    direct_video: bool,
    aspect_ratio_1: Option<AspectRatio>,
    aspect_ratio_2: Option<AspectRatio>,
    spi: &mut Spi<impl MemoryMapper>,
    is_menu: bool,
) -> Result<(), String> {
    private::select_mode(
        mode,
        direct_video,
        aspect_ratio_1,
        aspect_ratio_2,
        spi,
        is_menu,
    )
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

#[derive(Debug, Default, Copy, Clone)]
pub struct VideoInfo {
    resolution: Resolution,
    aspect_ratio: AspectRatio,

    // TODO: figure out what these timing numbers mean.
    htime_ms: u32,
    vtime_ms: u32,
    ptime_ms: u32,
    ctime_ms: u32,
    vtimeh: u32,

    pixerep: u16,
    de_h: u16,
    de_v: u16,

    arx: u16,
    ary: u16,
    arxy: u16,
    fb_en: u16,
    fb_fmt: u16,
    _fb_resolution: u16,
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
    fn read_video(&mut self, spi: &mut Spi<impl MemoryMapper>) -> Result<(), String> {
        let mut command = spi.command(UserIoCommands::UserIoGetVres);
        let new_res = command.get();

        self.res = new_res;
        self.resolution = Resolution::new(command.get_32() as u16, command.get_32() as u16);
        self.htime_ms = command.get_32();
        self.vtime_ms = command.get_32();
        self.ptime_ms = command.get_32();
        self.vtimeh = command.get_32();
        self.ctime_ms = command.get_32();
        self.interlaced = (new_res & 0x100) != 0;
        self.rotated = (new_res & 0x200) != 0;

        self.pixerep = command.get();
        self.de_h = command.get();
        self.de_v = command.get();

        Ok(())
    }

    fn read_fb_param(&mut self, spi: &mut Spi<impl MemoryMapper>) -> Result<(), String> {
        let mut command = spi.command_read(UserIoCommands::UserIoGetFbParams, &mut self.fb_crc);

        self.arx = command.get();
        self.arxy = !!(self.arx & 0x1000);
        self.arx &= 0xFFF;

        self.ary = command.get();
        self.ary &= 0xFFF;

        self.fb_fmt = command.get();
        self.fb_width = command.get();
        self.fb_height = command.get();
        self.fb_en = !!(self.fb_fmt & 0x40);

        Ok(())
    }

    /// Create a video info from the FPGA.
    pub(crate) fn create(spi: &mut Spi<impl MemoryMapper>) -> Result<Self, String> {
        let mut result = VideoInfo::default();
        result.read_video(spi)?;
        result.read_fb_param(spi)?;

        Ok(result)
    }

    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    pub fn fb_resolution(&self) -> Resolution {
        Resolution::new(self.fb_width, self.fb_height)
    }

    pub fn aspect_ratio(&self) -> AspectRatio {
        self.aspect_ratio
    }

    pub fn vtime(&self) -> Duration {
        Duration::from_nanos(self.vtime_ms as u64 * 10)
    }
}
