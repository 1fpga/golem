#![allow(unused)]

#[cfg(target_os = "linux")]
use i2cdev::core::I2CDevice;
use strum::{EnumString, FromRepr};
use tracing::{debug, info, trace, warn};

use cyclone_v::memory::MemoryMapper;

use crate::config::MisterConfig;
use crate::fpga::user_io::SetVideoMode;
use crate::fpga::Spi;

pub struct Edid {
    inner: [u8; 256],
}

impl Edid {
    fn new(inner: [u8; 256]) -> Self {
        Edid { inner }
    }

    pub fn into_inner(self) -> [u8; 256] {
        self.inner
    }

    #[cfg(target_os = "linux")]
    pub fn from_i2c() -> Result<Self, String> {
        let mut i2c = create_i2c("/dev/i2c-1", 0x39, false)?;

        // Test if adv7513 senses hdmi clock. If not, don't bother with the edid query.
        let hpd_state = i2c.smbus_read_byte_data(0x42).map_err(|e| e.to_string())?;
        if hpd_state & 0x20 == 0 {
            return Err("EDID: HDMI not connected.".to_string());
        }

        for _ in 0..10 {
            i2c.smbus_write_byte_data(0xC9, 0x03)
                .map_err(|e| e.to_string())?;
            i2c.smbus_write_byte_data(0xC9, 0x13)
                .map_err(|e| e.to_string())?;
        }

        let mut i2c = create_i2c("/dev/i2c-1", 0x3f, false)?;

        // waiting for valid EDID
        for _ in 0..20 {
            let edid: [u8; 256] = (0..256)
                .map(|i| i2c.smbus_read_byte_data(i as u8).map_err(|e| e.to_string()))
                .collect::<Result<Vec<_>, String>>()?
                .try_into()
                .unwrap();

            if is_edid_valid(&edid) {
                info!("EDID Info:\n{}", hexdump(&edid));
                return Ok(Self::new(edid));
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        Err("EDID: No valid EDID header found.".to_string())
    }
}

fn is_edid_valid(edid: &[u8]) -> bool {
    &edid[..8] == &[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]
}

#[cfg(target_os = "linux")]
fn create_i2c(
    path: impl AsRef<std::path::Path>,
    address: u16,
    is_smbus: bool,
) -> Result<i2cdev::linux::LinuxI2CDevice, String> {
    let mut i2c =
        i2cdev::linux::LinuxI2CDevice::new(path.as_ref(), address).map_err(|e| e.to_string())?;
    if is_smbus {
        i2c.smbus_write_quick(false).map_err(|e| e.to_string())?;
    } else {
        if let Err(e) = i2c.smbus_read_byte().map_err(|e| e.to_string()) {
            return Err(format!(
                "Unable to detect I2C device on bus {:?} (address 0x{:X}): {}",
                path.as_ref(),
                address,
                e
            ));
        }
    }

    Ok(i2c)
}

fn get_active_edid_() -> Result<[u8; 256], String> {
    #[cfg(target_os = "linux")]
    {
        Edid::from_i2c().map(|edid| edid.into_inner())
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err("EDID: Not supported on this platform.".to_string())
    }
}

fn get_edid_vmode_(options: &MisterConfig) -> Option<CustomVideoMode> {
    let edid = match get_active_edid_() {
        Ok(edid) => edid,
        Err(e) => {
            warn!("EDID Err while getting active edid: {}\n", e);
            return None;
        }
    };

    match parse_edid_vmode_(options, &edid) {
        Ok(vmode) => {
            if vmode.param.hact > 2048 {
                warn!(
                    "EDID: Preferred resolution is too high ({}x{}).",
                    vmode.param.hact, vmode.param.vact
                );
                return None;
            }

            Some(vmode)
        }
        Err(e) => {
            warn!("EDID Err parsing: {}\n", e);
            None
        }
    }
}

#[cfg(target_os = "linux")]
pub fn hdmi_config_set_spd(val: bool) -> Result<(), String> {
    let mut i2c = create_i2c("/dev/i2c-1", 0x39, false)?;

    let mut packet_val = i2c.smbus_read_byte_data(0x40).map_err(|e| e.to_string())?;
    if val {
        packet_val |= 0x40;
    } else {
        packet_val &= !0x40;
    }

    let res = i2c.smbus_write_byte_data(0x40, packet_val);
    match res {
        Ok(_) => Ok(()),
        Err(e) => {
            let e = e.to_string();
            warn!(
                "i2c: spd write error ({:02X} {:02X}): {}\n",
                0x40, packet_val, e
            );
            Err(e)
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn hdmi_config_set_spd(_val: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn hdmi_config_set_spare(_packet: bool, _enabled: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "linux")]
fn hdmi_config_set_spare(packet: bool, enabled: bool) -> Result<(), String> {
    let mut i2c = create_i2c("/dev/i2c-1", 0x39, false)?;
    let mask: u8 = if packet { 2 } else { 1 };

    let packet_val = i2c.smbus_read_byte_data(0x40).map_err(|e| e.to_string())?;
    let packet_val = if enabled {
        packet_val | mask
    } else {
        packet_val & !mask
    };

    let res = i2c.smbus_write_byte_data(0x40, packet_val);
    match res {
        Ok(_) => Ok(()),
        Err(e) => {
            let e = e.to_string();
            warn!(
                "i2c: spare write error ({:02X} {:02X}): {}\n",
                0x40, packet_val, e
            );
            Err(e)
        }
    }
}

fn parse_edid_vmode_(options: &MisterConfig, edid: &[u8]) -> Result<CustomVideoMode, String> {
    let pixel_clock_khz = ((edid[0x36] as u32 + ((edid[0x37] as u32) << 8)) * 10) as f64;

    if pixel_clock_khz < 10000. {
        return Err(format!(
            "Invalid EDID: Pixelclock < 10 MHz, assuming invalid data 0x{:02X} 0x{:02X}.\n",
            edid[0x36], edid[0x37],
        ));
    }

    if options.dvi_mode_raw().is_none() {
        let dvi_mode = if edid[0x80] == 2 && edid[0x81] == 3 && ((edid[0x83] & 0x40) != 0) {
            0
        } else {
            1
        };

        if dvi_mode == 1 {
            debug!("EDID: using DVI mode.");
        }
    }

    let x = &edid[0x36..];
    let flags = x[17];
    if flags & 0x80 != 0 {
        return Err(
            "EDID: preferred mode is interlaced. Fall back to default video mode.".to_string(),
        );
    }

    let hact: u32 = x[2] as u32 + ((x[4] as u32 & 0xf0) << 4);
    let hbl: u32 = x[3] as u32 + ((x[4] as u32 & 0x0f) << 8);
    let hfp: u32 = x[8] as u32 + ((x[11] as u32 & 0xc0) << 2);
    let hsync: u32 = x[9] as u32 + ((x[11] as u32 & 0x30) << 4);
    let hbp: u32 = hbl - hsync - hfp;
    let vact: u32 = x[5] as u32 + ((x[7] as u32 & 0xf0) << 4);
    let vbl: u32 = x[6] as u32 + ((x[7] as u32 & 0x0f) << 8);
    let vfp: u32 = (x[10] as u32 >> 4) + ((x[11] as u32 & 0x0c) << 2);
    let vsync: u32 = (x[10] as u32 & 0x0f) + ((x[11] as u32 & 0x03) << 4);
    let vbp: u32 = vbl - vsync - vfp;

    let f_pix = pixel_clock_khz / 1000.;
    let mut v = CustomVideoMode::default();
    v.param.mode = 0;
    v.param.hact = hact;
    v.param.hfp = hfp;
    v.param.hs = hsync;
    v.param.hbp = hbp;
    v.param.vact = vact;
    v.param.vfp = vfp;
    v.param.vs = vsync;
    v.param.vbp = vbp;
    v.f_pix = f_pix;

    let frame_rate = v.frame_rate();
    debug!(
        "EDID: preferred mode: {}x{}@{:.1}, pixel clock: {:.3}MHz",
        hact, vact, frame_rate, f_pix
    );

    if f_pix > 210. {
        warn!(
            "EDID: Preferred mode has too high pixel clock ({:.3}MHz).",
            f_pix
        );

        if hact == 2048 && vact == 1536 {
            let mode = DefaultVideoMode::V2048x1536r60;
            warn!("EDID: Using safe vmode ({:?}).", mode);

            v.param = mode.into();
            v.param.vic = mode.vic_mode();
            v.f_pix = mode.f_pix();
        } else if frame_rate > 60. {
            let f_pix =
                60. * (((hact + hfp + hbp + hsync) * (vact + vfp + vbp + vsync)) as f64) / 1000000.;
            if f_pix <= 210. {
                warn!(
                    "EDID: Reducing frame rate to 60Hz with new pixel clock {:.3}MHz.",
                    f_pix
                );
                v.f_pix = f_pix;
            } else {
                return Err("EDID: Falling back to default video mode.".to_string());
            }
        } else {
            return Err("EDID: Falling back to default video mode.".to_string());
        }
    }

    v.param.rb = 2;
    v.set_pll(v.f_pix);
    Ok(v)
}

fn hexdump(data: &[u8]) -> String {
    let mut n = 0;
    let mut size = data.len();
    let mut s = String::new();

    while size > 0 {
        s.push_str(&format!("{:04x}: ", n));

        let b2c = size.min(16);
        let sub = &data[n..n + b2c];
        sub.iter().for_each(|b| {
            s.push(b"0123456789abcdef"[(*b >> 4) as usize] as char);
            s.push(b"0123456789abcdef"[(*b & 0x0F) as usize] as char);
            s.push(' ');
        });
        s.push(' ');
        s.push(' ');

        for _ in 0..(16 - b2c.max(16)) {
            s.push_str("   ");
        }

        sub.iter().for_each(|b| {
            s.push(if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                '.'
            });
        });

        size -= b2c;
        n += b2c;
        s.push('\n');
    }

    s
}

#[derive(Debug, Default, Clone, Copy)]
struct Pll {
    pub c: u32,
    pub m: u32,
    pub ko: f64,
}

fn get_pll_div_(div: u32) -> u32 {
    if div & 1 != 0 {
        0x20000 | (((div / 2) + 1) << 8) | (div / 2)
    } else {
        ((div / 2) << 8) | (div / 2)
    }
}

fn find_pll_once_(f_out: f64, mut c: u32) -> Option<Pll> {
    while (f_out * c as f64) < 400. {
        c += 1;
    }

    let fvco = f_out * c as f64;
    let m = (fvco / 50.) as u32;
    let ko = (fvco / 50.) - m as f64;

    Some(Pll { c, m, ko })
}

fn find_pll_par_(f_out: f64) -> Option<Pll> {
    let mut c = 1;
    loop {
        let Pll { c: c_2, m, ko } = find_pll_once_(f_out, c)?;
        c = c_2;
        let fvco = (ko + m as f64) * 50.;

        if ko <= 0.05 || ko >= 0.95 {
            debug!("Fvco={}, C={}, M={}, K={}", fvco, c, m, ko);
            if fvco > 1500. {
                debug!("-> No exact parameters found");
                return None;
            }

            trace!("-> K is outside allowed range");
            c += 1;
        } else {
            return Some(Pll { c, m, ko });
        }
    }
}

#[derive(Debug, Clone, Copy, FromRepr, EnumString)]
#[repr(u8)]
pub enum DefaultVideoMode {
    V1280x720r60 = 0,
    V1024x768r60,
    V720x480r60,
    V720x576r50,
    V1280x1024r60,
    V800x600r60,
    V640x480r60,
    V1280x720r50,
    V1920x1080r60,
    V1920x1080r50,
    V1366x768r60,
    V1024x600r60,
    V1920x1440r60,
    V2048x1536r60,
    V2560x1440r60,

    Ntsc15K,
    Ntsc31K,
    Pal15k,
    Pal31k,
}

impl DefaultVideoMode {
    pub fn v_param(&self) -> &'static [u32; 8] {
        #[rustfmt::skip]
        const V_PARAM_DEFAULT_MODES: [[u32; 8]; 19] = [
            [1280, 110, 40, 220, 720, 5, 5, 20],  //  0  1280x 720@60
            [1024, 24, 136, 160, 768, 3, 6, 29],  //  1  1024x 768@60
            [720, 16, 62, 60, 480, 9, 6, 30],     //  2   720x 480@60
            [720, 12, 64, 68, 576, 5, 5, 39],     //  3   720x 576@50
            [1280, 48, 112, 248, 1024, 1, 3, 38], //  4  1280x1024@60
            [800, 40, 128, 88, 600, 1, 4, 23],    //  5   800x 600@60
            [640, 16, 96, 48, 480, 10, 2, 33],    //  6   640x 480@60
            [1280, 440, 40, 220, 720, 5, 5, 20],  //  7  1280x 720@50
            [1920, 88, 44, 148, 1080, 4, 5, 36],  //  8  1920x1080@60
            [1920, 528, 44, 148, 1080, 4, 5, 36], //  9  1920x1080@50
            [1366, 70, 143, 213, 768, 3, 3, 24],  // 10  1366x 768@60
            [1024, 40, 104, 144, 600, 1, 3, 18],  // 11  1024x 600@60
            [1920, 48, 32, 80, 1440, 2, 4, 38],   // 12  1920x1440@60
            [2048, 48, 32, 80, 1536, 2, 4, 38],   // 13  2048x1536@60
            [1280, 24, 16, 40, 1440, 3, 5, 33],   // 14  2560x1440@60 (pr)

            // TV modes.
            [640, 30, 60, 70, 240, 4, 4, 14], // NTSC 15K
            [640, 16, 96, 48, 480, 8, 4, 33], // NTSC 31K
            [640, 30, 60, 70, 288, 6, 4, 14], //  PAL 15K
            [640, 16, 96, 48, 576, 2, 4, 42], //  PAL 31K
        ];

        V_PARAM_DEFAULT_MODES.get(*self as usize).unwrap()
    }

    pub fn f_pix(&self) -> f64 {
        *[
            74.25,   //  0  1280x 720@60
            65.,     //  1  1024x 768@60
            27.,     //  2   720x 480@60
            27.,     //  3   720x 576@50
            108.,    //  4  1280x1024@60
            40.,     //  5   800x 600@60
            25.175,  //  6   640x 480@60
            74.25,   //  7  1280x 720@50
            148.5,   //  8  1920x1080@60
            148.5,   //  9  1920x1080@50
            85.5,    // 10  1366x 768@60
            48.96,   // 11  1024x 600@60
            185.203, // 12  1920x1440@60
            209.318, // 13  2048x1536@60
            120.75,  // 14  2560x1440@60 (pr)
            12.587,  // NTSC 15K
            25.175,  // NTSC 31K
            12.587,  //  PAL 15K
            25.175,  //  PAL 31K
        ]
        .get(*self as usize)
        .unwrap()
    }

    pub fn vic_mode(&self) -> u32 {
        *[
            4, //0  1280x720@60
            0, //1  1024x768@60
            3, //2  720x480@60
            8, //3  720x576@50
            0, //4  1280x1024@60
            0, //5  800x600@60
            1, //6  640x480@60
            9, //7  1280x720@50
            6, //8  1920x1080@60
            1, //9  1920x1080@50
            0, //10 1366x768@60
            0, //11 1024x600@60
            0, //12 1920x1440@60
            0, //13 2048x1536@60
            0, //14 2560x1440@60 (pr)
            0, // NTSC 15K
            0, // NTSC 31K
            0, //  PAL 15K
            0, //  PAL 31K
        ]
        .get(*self as usize)
        .unwrap()
    }
}

impl From<DefaultVideoMode> for CustomVideoModeParam {
    fn from(mode: DefaultVideoMode) -> Self {
        let vpar = mode.v_param();
        let mode = mode as u8 as u32;
        CustomVideoModeParam {
            mode,
            hact: vpar[0],
            hfp: vpar[1],
            hs: vpar[2],
            hbp: vpar[3],
            vact: vpar[4],
            vfp: vpar[5],
            vs: vpar[6],
            vbp: vpar[7],
            pll: [0; 12],
            hpol: 0,
            vpol: 0,
            vic: 0,
            rb: 0,
            pr: 0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CustomVideoModeParam {
    pub mode: u32, // 0

    pub hact: u32, // 1
    pub hfp: u32,  // 2
    pub hs: u32,   // 3
    pub hbp: u32,  // 4

    pub vact: u32, // 5
    pub vfp: u32,  // 6
    pub vs: u32,   // 7
    pub vbp: u32,  // 8

    pub pll: [u32; 12], // 9-20

    // These are polarity for hsync and vsync.
    // Not sure why hs and vs cannot be 2-complement (and thus `i32`).
    pub hpol: u32, // 21
    pub vpol: u32, // 22

    pub vic: u32, // 23
    pub rb: u32,  // 24
    pub pr: u32,  // 25
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CustomVideoMode {
    pub param: CustomVideoModeParam,

    pub vrr: bool,
    pub f_pix: f64,
}

impl CustomVideoMode {
    pub fn send_to_core(
        &self,
        direct_video: bool,
        spi: &mut Spi<impl MemoryMapper>,
        is_menu: bool,
    ) -> Result<(), String> {
        let mut fixed = *self;

        // TODO: This doesn't make sense and should be done in another place.
        // At this point in the process if all your options aren't processed you're
        // doing something wrong.
        // No need to have options passed to this function.
        if direct_video {
            fixed.param.hfp = 6;
            fixed.param.hbp = 3;
            fixed.param.hact += self.param.hfp - fixed.param.hfp;
            fixed.param.hact += self.param.hbp - fixed.param.hbp;

            fixed.param.vfp = 2;
            fixed.param.vbp = 2;
            fixed.param.hact += self.param.vfp - fixed.param.vfp;
            fixed.param.hact += self.param.vbp - fixed.param.vbp;
        } else {
            hdmi_config_set_spd(false)?;
            hdmi_config_set_spare(false, false)?;
        }

        if !is_menu {
            info!("Sending HDMI parameters...");
            spi.execute(SetVideoMode(&fixed))?;
        }

        Ok(())
    }

    pub fn set_pll(&mut self, f_out: f64) {
        trace!("Calculate PLL for {:.4} MHz", f_out);

        let Pll { c, m, ko } = find_pll_par_(f_out).unwrap_or_else(|| {
            let Pll { c, mut m, mut ko } = find_pll_once_(f_out, 1).unwrap_or_default();

            //Make sure K is in allowed range.
            if ko <= 0.05 {
                ko = 0.;
            } else if ko >= 0.95 {
                m += 1;
                ko = 0.;
            }

            Pll { c, m, ko }
        });

        let k: u32 = if ko >= 0.05 {
            (ko * 4294967296f64) as u32
        } else {
            1
        };

        let fvco = (ko + m as f64) * 50.;
        let f_pix = fvco / (c as f64);

        debug!(
            "Fvco={}, C={}, M={}, K={}({}) -> Fpix={}",
            fvco, c, m, ko, k, f_pix
        );

        self.param.pll = [
            4,
            get_pll_div_(m),
            3,
            0x10000,
            5,
            get_pll_div_(c),
            9,
            2,
            8,
            7,
            7,
            k,
        ];
        self.f_pix = f_pix;
    }

    pub fn frame_rate(&self) -> f64 {
        let p = &self.param;
        self.f_pix * 1000000.
            / (((p.hact + p.hfp + p.hbp + p.hs) * (p.vact + p.vfp + p.vbp + p.vs)) as f64)
    }
}

impl From<DefaultVideoMode> for CustomVideoMode {
    fn from(mode: DefaultVideoMode) -> Self {
        let mut v = CustomVideoMode::default();
        v.param = mode.into();
        v.f_pix = mode.f_pix();
        v.set_pll(v.f_pix);
        v
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct VideoModeDef {
    pub vmode_def: Option<CustomVideoMode>,
    pub vmode_pal: Option<CustomVideoMode>,
    pub vmode_ntsc: Option<CustomVideoMode>,
}

fn parse_custom_video_mode(video_mode: Option<&str>) -> CustomVideoMode {
    if video_mode.is_none() || matches!(video_mode, Some("auto")) || matches!(video_mode, Some(""))
    {
        return DefaultVideoMode::V1920x1080r60.into();
        // return DefaultVideoMode::V1280x720r60.into();
        // return DefaultVideoMode::V640x480r60.into();
    }
    DefaultVideoMode::V1920x1080r60.into()

    // todo!("parse_custom_video_mode")
}

pub fn select_video_mode(options: &MisterConfig) -> Result<VideoModeDef, String> {
    if options.direct_video() {
        let mode = match (options.menu_pal(), options.forced_scandoubler()) {
            (false, false) => DefaultVideoMode::Ntsc15K,
            (false, true) => DefaultVideoMode::Ntsc31K,
            (true, false) => DefaultVideoMode::Pal15k,
            (true, true) => DefaultVideoMode::Pal31k,
        };

        let mut vpar: CustomVideoMode = mode.into();
        vpar.set_pll(mode.f_pix());

        Ok(VideoModeDef {
            vmode_def: Some(vpar),
            vmode_pal: None,
            vmode_ntsc: None,
        })
    } else {
        eprintln!("select_video_mode: conf: {:?}", options.video_conf);
        // return Ok(VideoModeDef {
        //     vmode_def: Some(DefaultVideoMode::V640x480r60.into()),
        //     vmode_pal: None,
        //     vmode_ntsc: None,
        // });

        if options.video_conf.is_none()
            && options.video_conf_pal.is_none()
            && options.video_conf_ntsc.is_none()
        {
            if let Some(vmode) = get_edid_vmode_(options) {
                return Ok(VideoModeDef {
                    vmode_def: Some(vmode),
                    vmode_pal: None,
                    vmode_ntsc: None,
                });
            }
        }

        let def = parse_custom_video_mode(options.video_conf.as_deref());

        Ok(VideoModeDef {
            vmode_def: Some(def),
            vmode_pal: None,
            vmode_ntsc: None,
        })
    }
}

#[test]
fn parse_4k_hdmi_edid() {
    // This is the EDID from my monitor (VESA 4K).
    let edid = hex::decode(
        "\
        00 ff ff ff ff ff ff 00 14 e1 6a 00 00 00 00 00 \
        1b 1d 01 03 80 3c 22 78 0a da ff a3 58 4a a2 29 \
        17 49 4b a5 4f 00 d1 fc 81 bc 31 68 31 7c 45 68 \
        45 7c 61 68 61 7c 08 e8 00 30 f2 70 5a 80 b0 58 \
        8a 00 ba 88 21 00 00 1e 00 00 00 10 00 00 00 00 \
        00 00 00 00 00 00 00 00 00 00 00 00 00 fc 00 48 \
        44 36 30 20 53 2b 0a 20 20 20 20 20 00 00 00 fd \
        00 17 92 0f a0 3c 00 0a 20 20 20 20 20 20 01 0e \
        02 03 4a e2 57 90 61 04 03 02 01 60 1f 13 12 11 \
        5f 5e 5d 22 21 20 05 14 07 06 16 15 23 09 07 07 \
        83 01 00 00 6e 03 0c 00 10 00 38 3c 20 00 80 01 \
        02 03 04 67 d8 5d c4 01 78 80 03 e3 0f 42 38 e3 \
        05 e2 01 e6 06 07 01 00 00 00 56 5e 00 a0 a0 a0 \
        29 50 30 20 35 00 56 50 21 00 00 1e 00 00 00 00 \
        00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 \
        00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 93 \
        "
        .replace(' ', ""),
    )
    .unwrap();

    let vmode = parse_edid_vmode_(&MisterConfig::new_defaults(), &edid).unwrap();
    assert_eq!(vmode.param.hact, 3840);
    assert_eq!(vmode.param.vact, 2160);
    assert_eq!(vmode.frame_rate(), 60.);
}
