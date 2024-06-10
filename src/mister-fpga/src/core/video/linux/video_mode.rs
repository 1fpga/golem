use i2cdev::core::I2CDevice;
use tracing::{debug, error};

use cyclone_v::memory::MemoryMapper;

use crate::config;
use crate::config::aspect::AspectRatio;
use crate::config::edid::CustomVideoMode;
use crate::config::FramebufferSizeConfig;
use crate::fpga::user_io::{
    DisableGamma, EnableGamma, IsGammaSupported, SetCustomAspectRatio, SetFramebufferToCore,
    SetFramebufferToLinux,
};
use crate::fpga::Spi;

pub struct GammaConfiguration(Vec<(u8, u8, u8)>);

#[allow(dead_code)]
impl GammaConfiguration {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, r: u8, g: u8, b: u8) {
        self.0.push((r, g, b));
    }

    pub fn add_grayscale(&mut self, v: u8) {
        self.0.push((v, v, v));
    }

    pub fn set(&self, spi: &mut Spi<impl MemoryMapper>) -> Result<(), String> {
        if self.0.is_empty() {
            spi.execute(DisableGamma)?;
        } else {
            spi.execute(EnableGamma(self.0.as_slice()))?;
        }

        Ok(())
    }
}

fn video_fb_config(
    mode: &CustomVideoMode,
    fb_size: FramebufferSizeConfig,
    vscale_border: u16,
    direct_video: bool,
    spi: &mut Spi<impl MemoryMapper>,
    _is_menu: bool,
) -> Result<(), String> {
    let mut fb_scale = fb_size.as_scale() as u32;

    if fb_scale <= 1 {
        if mode.param.hact * mode.param.vact > 1920 * 1080 {
            fb_scale = 2;
        } else {
            fb_scale = 1;
        }
    }

    let fb_scale_x = fb_scale;
    let fb_scale_y = if mode.param.pr == 0 {
        fb_scale
    } else {
        fb_scale * 2
    };

    let width = (mode.param.hact / fb_scale_x) as u16;
    let height = (mode.param.vact / fb_scale_y) as u16;

    let brd_x = vscale_border / fb_scale_x as u16;
    let brd_y = vscale_border / fb_scale_y as u16;
    debug!("video_fb_config: fb_scale_x={}, fb_scale_y={}, fb_width={}, fb_height={}, brd_x={}, brd_y={}", fb_scale_x, fb_scale_y, width, height, brd_x, brd_y);

    let (x_offset, y_offset) = if direct_video {
        ((mode.param.hbp - 3) as u16, (mode.param.vbp - 2) as u16)
    } else {
        (0, 0)
    };

    spi.execute(SetFramebufferToLinux {
        n: 1,
        x_offset,
        y_offset,
        width,
        height,
        hact: mode.param.hact as u16,
        vact: mode.param.vact as u16,
    })?;

    Ok(())
}

fn hdmi_config_set_mode(
    direct_video: bool,
    mode: &config::video::edid::CustomVideoMode,
) -> Result<(), String> {
    let vic_mode = mode.param.vic as u8;
    let pr_flags = if direct_video {
        0 // Automatic Pixel Repetition.
    } else if mode.param.pr != 0 {
        0b01001000 // Manual Pixel Repetition with 2x clock.
    } else {
        0b01000000 // Manual Pixel Repetition.
    };

    let sync_invert = ((!mode.param.hpol as u8) << 5) | ((!mode.param.vpol as u8) << 6);

    #[rustfmt::skip]
        let init_data = [
        (0x17, (0b00000010 | sync_invert)), // Aspect ratio 16:9 [1]=1, 4:3 [1]=0
        (0x3B, pr_flags),
        (0x3C, vic_mode),                   // VIC
    ];

    let mut i2c = super::create_i2c(0x39)?;
    for (reg, value) in init_data.into_iter() {
        i2c.smbus_write_byte_data(reg, value)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn select_mode(
    mode: CustomVideoMode,
    fb_size: FramebufferSizeConfig,
    vscale_border: u16,
    direct_video: bool,
    aspect_ratio_1: Option<AspectRatio>,
    aspect_ratio_2: Option<AspectRatio>,
    spi: &mut Spi<impl MemoryMapper>,
    is_menu: bool,
) -> Result<(), String> {
    let mut has_gamma = false;
    spi.execute(IsGammaSupported(&mut has_gamma))?;

    if has_gamma {
        let mut gamma = GammaConfiguration::new();
        // TODO: get gamma configuration from options/database.
        gamma.add_grayscale(0);
        gamma.add_grayscale(0x7F);
        gamma.add_grayscale(0xFF);
        gamma.set(spi)?;
    }

    if aspect_ratio_1.or(aspect_ratio_2).is_some() {
        let first = aspect_ratio_1.unwrap_or_else(AspectRatio::zero);
        let second = aspect_ratio_2.unwrap_or_else(AspectRatio::zero);

        spi.execute(SetCustomAspectRatio(first.into(), second.into()))?;
    }

    // TODO: set scaler filter.
    // TODO: set VRR.

    mode.send_to_core(direct_video, spi, is_menu)?;
    if is_menu {
        hdmi_config_set_mode(direct_video, &mode)?;
        video_fb_config(&mode, fb_size, vscale_border, direct_video, spi, is_menu)?;
    } else {
        spi.execute(SetFramebufferToCore)?;
    }
    Ok(())
}

pub fn init_mode(
    options: &config::MisterConfig,
    spi: &mut Spi<impl MemoryMapper>,
    is_menu: bool,
) -> Result<(), String> {
    let mode = config::video::edid::select_video_mode(options)?;

    let Some(m) = mode.vmode_def else {
        error!("No video mode selected");
        return Err("No video mode selected".to_string());
    };
    eprintln!("Selected video mode: {:?}", m);

    select_mode(
        m,
        options.fb_size.unwrap_or_default(),
        options.vscale_border.unwrap_or_default(),
        options.direct_video(),
        options.custom_aspect_ratio().first().cloned(),
        options.custom_aspect_ratio().get(1).cloned(),
        spi,
        is_menu,
    )?;

    Ok(())
}
