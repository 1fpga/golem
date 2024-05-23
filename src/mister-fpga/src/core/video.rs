use crate::config;
use crate::config::aspect::AspectRatio;
use crate::config::edid::CustomVideoMode;
use crate::config::resolution::Resolution;
use crate::fpga::user_io::UserIoCommands;
use crate::fpga::Spi;
use cyclone_v::memory::MemoryMapper;
use std::time::Duration;
use tracing::{debug, error, info, warn};

mod linux;

/// Initialize the video Hardware configuration.
// TODO: this should not take the whole config but a subset of it related only to video.
pub fn init(options: &config::MisterConfig) {
    if let Err(error) = linux::hdmi_config_init(options) {
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
    linux::select_mode(
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
    fpga: &mut crate::fpga::MisterFpga,
    is_menu: bool,
) {
    if is_menu {
        info!("Initializing video mode for menu");

        if let Err(error) = linux::init_mode_menu(options, fpga) {
            error!("Failed to initialize video mode: {}", error);
            warn!("This is not a fatal error, the application will continue to run.");
        }
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

        command.write_read(0, &mut self.res);
        let mut resx = 0;
        let mut resy = 0;
        command
            .write_read_32(0, 0, &mut resx)
            .write_read_32(0, 0, &mut resy);
        self.resolution = Resolution::new(resx as u16, resy as u16);
        self.aspect_ratio = self.resolution.aspect_ratio();
        command
            .write_read_32(0, 0, &mut self.htime_ms)
            .write_read_32(0, 0, &mut self.vtime_ms)
            .write_read_32(0, 0, &mut self.ptime_ms)
            .write_read_32(0, 0, &mut self.vtimeh)
            .write_read_32(0, 0, &mut self.ctime_ms)
            .write_read(0, &mut self.pixerep)
            .write_read(0, &mut self.de_h)
            .write_read(0, &mut self.de_v);

        self.interlaced = (self.res & 0x100) != 0;
        self.rotated = (self.res & 0x200) != 0;
        Ok(())
    }

    fn read_fb_param(&mut self, spi: &mut Spi<impl MemoryMapper>) -> Result<(), String> {
        let mut command = spi.command_read(UserIoCommands::UserIoGetFbParams, &mut self.fb_crc);

        command.write_read(0, &mut self.arx);
        self.arxy = !!(self.arx & 0x1000);
        self.arx &= 0xFFF;
        command.write_read(0, &mut self.ary);
        self.ary &= 0xFFF;

        command.write_read(0, &mut self.fb_fmt);
        command.write_read(0, &mut self.fb_width);
        command.write_read(0, &mut self.fb_height);
        self.fb_en = !!(self.fb_fmt & 0x40);
        Ok(())
    }

    /// Create a video info from the FPGA.
    pub(crate) fn create(spi: &mut Spi<impl MemoryMapper>) -> Result<Self, String> {
        let mut result = VideoInfo::default();
        result.read_video(spi)?;
        result.read_fb_param(spi)?;

        debug!("VideoInfo: {:?}", result);
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
