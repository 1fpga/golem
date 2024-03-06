use crate::config;
use crate::config::aspect::AspectRatio;
use crate::fpga::user_io::{
    DisableGamma, EnableGamma, IsGammaSupported, SetCustomAspectRatio, SetFramebufferToCore,
    SetFramebufferToLinux,
};
use crate::fpga::Spi;
use cyclone_v::memory::MemoryMapper;
use i2cdev::core::I2CDevice;
use tracing::{debug, error};

pub struct GammaConfiguration(Vec<(u8, u8, u8)>);

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
    options: &config::MisterConfig,
    mode: &config::video::edid::CustomVideoMode,
    spi: &mut Spi<impl MemoryMapper>,
    _is_menu: bool,
) -> Result<(), String> {
    let mut fb_scale = options.fb_size.unwrap_or_default().as_scale() as u32;

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

    let brd_x = options.vscale_border.unwrap_or_default() / fb_scale_x as u16;
    let brd_y = options.vscale_border.unwrap_or_default() / fb_scale_y as u16;
    debug!("video_fb_config: fb_scale_x={}, fb_scale_y={}, fb_width={}, fb_height={}, brd_x={}, brd_y={}", fb_scale_x, fb_scale_y, width, height, brd_x, brd_y);

    let xoff = if options.direct_video() {
        mode.param.hbp - 3
    } else {
        0
    } as u16;
    let yoff = if options.direct_video() {
        mode.param.vbp - 2
    } else {
        0
    } as u16;

    spi.execute(SetFramebufferToLinux {
        n: 1,
        xoff,
        yoff,
        width,
        height,
        hact: mode.param.hact as u16,
        vact: mode.param.vact as u16,
    })?;

    Ok(())
}

fn hdmi_config_set_mode(
    options: &config::MisterConfig,
    mode: &config::video::edid::CustomVideoMode,
) -> Result<(), String> {
    let vic_mode = mode.param.vic as u8;
    let pr_flags = if options.direct_video() {
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

pub fn init_mode(
    options: &config::MisterConfig,
    spi: &mut Spi<impl MemoryMapper>,
    is_menu: bool,
) -> Result<(), String> {
    let mode = config::video::edid::select_video_mode(options)?;
    eprintln!("Selected video mode: {:?}", mode);

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

    let arc = options.custom_aspect_ratio();
    if !arc.is_empty() {
        let first = arc.first().copied().unwrap_or_else(AspectRatio::zero);
        let second = arc.get(1).copied().unwrap_or_else(AspectRatio::zero);

        spi.execute(SetCustomAspectRatio(first.into(), second.into()))?;
    }

    // TODO: set scaler filter.
    // TODO: set VRR.

    if let Some(m) = mode.vmode_def.clone() {
        m.send_to_core(options, spi, is_menu)?;

        if is_menu {
            hdmi_config_set_mode(options, &m)?;
            video_fb_config(options, &m, spi, is_menu)?;
        } else {
            spi.execute(SetFramebufferToCore)?;
        }
    } else {
        error!("No video mode defined");
    }

    Ok(())
}
