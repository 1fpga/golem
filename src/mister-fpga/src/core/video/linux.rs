use glam::Mat4;
use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;
use tracing::{debug, error};

use cyclone_v::memory::MemoryMapper;

use crate::config;
use crate::config::{HdmiLimitedConfig, HdrConfig, MisterConfig, VgaMode, video};
use crate::config::aspect::AspectRatio;
use crate::config::edid::CustomVideoMode;
use crate::fpga::Spi;

mod video_mode;

/// Just to simplify creating matrices.
macro_rules! matrix {
    ($($x:expr),* $(,)?) => {
        Mat4::from_cols_array(&[$($x as f32),*])
    };
}

fn create_i2c(address: u16) -> Result<LinuxI2CDevice, String> {
    let i2c = LinuxI2CDevice::new("/dev/i2c-1", address).map_err(|e| e.to_string())?;
    // i2c.set_smbus_pec(true).map_err(|e| e.to_string())?;
    Ok(i2c)
}

fn send_to_i2c(i2c: &mut LinuxI2CDevice, data: &[(u8, u8)]) -> Result<(), String> {
    for (reg, val) in data {
        if let Err(e) = i2c.smbus_write_byte_data(*reg, *val) {
            error!(?e, "Failed to write HDMI register");
            return Err(e.to_string());
        }
    }

    Ok(())
}

pub fn hdmi_config_init(options: &MisterConfig) -> Result<(), String> {
    debug!(?options, "Initializing HDMI configuration");
    let mut i2c = create_i2c(0x39)?;

    let init_data: &[(u8, u8)] = &[
        // ADI required Write
        (0x98, 0x03),
        // [7:6] HPD Control...
        // 00 = HPD is from both HPD pin or CDC HPD
        // 01 = HPD is from CDC HPD
        // 10 = HPD is from HPD pin
        // 11 = HPD is always high
        (0xD6, 0b11000000),
        // Power Down control
        (0x41, 0x10),
        // ADI required Write.
        (0x9A, 0x70),
        // ADI required Write.
        (0x9C, 0x30),
        // [7:4] must be b0110!.
        // [3:2] 0b00 = Input clock not divided.
        //       0b01 = Clk divided by 2.
        //       0b10 = Clk divided by 4.
        //       0b11 = invalid!
        // [1:0] must be b01!
        (0x9D, 0b01100001),
        // ADI required Write.
        (0xA2, 0xA4),
        // ADI required Write.
        (0xA3, 0xA4),
        // ADI required Write.
        (0xE0, 0xD0),
        (0x35, 0x40),
        (0x36, 0xD9),
        (0x37, 0x0A),
        (0x38, 0x00),
        (0x39, 0x2D),
        (0x3A, 0x00),
        // Output Format 444 [7]=0.
        // [6] must be 0!
        // Colour Depth for Input Video data [5:4] b11 = 8-bit.
        // Input Style [3:2] b10 = Style 1 (ignored when using 444 input).
        // DDR Input Edge falling [1]=0 (not using DDR atm).
        // Output Colour Space RGB [0]=0.
        (0x16, 0b00111000),
        // Aspect ratio 16:9 [1]=1, 4:3 [1]=0, invert sync polarity
        (0x17, 0b01100010),
        // Automatic pixel repetition and VIC detection
        (0x3B, 0x0),
        // [6]=0 Normal bus order!
        // [5] DDR Alignment.
        // [4:3] b01 Data right justified (for YCbCr 422 input modes).
        (0x48, 0b00001000),
        // ADI required Write.
        (0x49, 0xA8),
        //Auto-Calculate SPD checksum
        (0x4A, 0b10000000),
        // ADI required Write.
        (0x4C, 0x00),
        // [7] must be 0!. Set RGB444 in AVinfo Frame [6:5], Set active format [4].
        // AVI InfoFrame Valid [4].
        // Bar Info [3:2] b00 Invalid. 0b01 Bars vertical. 0b10 Horizontal. 0b11 Both.
        // Scan Info [1:0] b00 (No data). 0b01 TV. b10 PC. b11 None.
        (
            0x55,
            (if options.hdmi_game_mode() {
                0b00010010
            } else {
                0b00010000
            }),
        ),
        // [5:4] Picture Aspect Ratio
        // [3:0] Active Portion Aspect Ratio b1000 = Same as Picture Aspect Ratio
        (
            0x56,
            (0b00001000
                + if options.hdr().is_enabled() {
                0b11000000
            } else {
                0
            }),
        ),
        // [7] IT Content. 0 - No. 1 - Yes (type set in register 0x59).
        // [6:4] Color space (ignored for RGB)
        // [3:2] RGB Quantization range
        // [1:0] Non-Uniform Scaled: 00 - None. 01 - Horiz. 10 - Vert. 11 - Both.
        (
            0x57,
            (if options.hdmi_game_mode() { 0x80 } else { 0 })
                | if options.vga_mode() == VgaMode::Ypbpr || options.hdmi_limited().is_limited() {
                0b0000100
            } else if options.hdr().is_enabled() {
                0b1101000
            } else {
                0b0001000
            },
        ),
        // [7:6] [YQ1 YQ0] YCC Quantization Range: b00 = Limited Range, b01 = Full Range
        // [5:4] IT Content Type b11 = Game, b00 = Graphics/None
        // [3:0] Pixel Repetition Fields b0000 = No Repetition
        (0x59, if options.hdmi_game_mode() { 0x30 } else { 0 }),
        (0x73, 0x01),
        // [7]=1 HPD Interrupt Enabled.
        (0x94, 0b10000000),
        // ADI required Write.
        (0x99, 0x02),
        // ADI required Write.
        (0x9B, 0x18),
        // ADI required Write.
        (0x9F, 0x00),
        // [6]=1 Monitor Sense Power Down DISabled.
        (0xA1, 0b00000000),
        // ADI required Write.
        (0xA4, 0x08),
        // ADI required Write.
        (0xA5, 0x04),
        // ADI required Write.
        (0xA6, 0x00),
        // ADI required Write.
        (0xA7, 0x00),
        // ADI required Write.
        (0xA8, 0x00),
        // ADI required Write.
        (0xA9, 0x00),
        // ADI required Write.
        (0xAA, 0x00),
        // ADI required Write.
        (0xAB, 0x40),
        // [7]=0 HDCP Disabled.
        // [6:5] must be b00!
        // [4]=0 Current frame is unencrypted
        // [3:2] must be b01!
        // [1]=1 HDMI Mode.
        // [0] must be b0!
        (
            0xAF,
            (0b00000100 | (if options.dvi_mode() { 0 } else { 0b10 })),
        ),
        // ADI required Write.
        (0xB9, 0x00),
        // [7:5] Input Clock delay...
        // b000 = -1.2ns.
        // b001 = -0.8ns.
        // b010 = -0.4ns.
        // b011 = No delay.
        // b100 = 0.4ns.
        // b101 = 0.8ns.
        // b110 = 1.2ns.
        // b111 = 1.6ns.
        (0xBA, 0b01100000),
        // ADI required Write.
        (0xBB, 0x00),
        // ADI required Write.
        (0xDE, 0x9C),
        // ADI required Write.
        (0xE4, 0x60),
        // Nbr of times to search for good phase
        // (Audio stuff on Programming Guide, Page 66)...
        (0xFA, 0x7D),
        // [6:4] Audio Select. 0b000 = I2S.
        // [3:2] Audio Mode. (HBR stuff, leave at 00!).
        (0x0A, 0b00000000),
        //
        (0x0B, 0b00001110),
        // [7] 0 = Use sampling rate from I2S stream.   1 = Use samp rate from I2C Register.
        // [6] 0 = Use Channel Status bits from stream. 1 = Use Channel Status bits from I2C register.
        // [2] 1 = I2S0 Enable.
        // [1:0] I2S Format: 00 = Standard. 01 = Right Justified. 10 = Left Justified. 11 = AES.
        (0x0C, 0b00000100),
        // [4:0] I2S Bit (Word) Width for Right-Justified.
        (0x0D, 0b00010000),
        // [3:0] Audio Word Length. 0b0010 = 16 bits.
        (0x14, 0b00000010),
        // I2S Sampling Rate [7:4]. b0000 = (44.1KHz). b0010 = 48KHz.
        // Input ID [3:1] b000 (0) = 24-bit RGB 444 or YCrCb 444 with Separate Syncs.
        (
            0x15,
            0b0100000 | if options.hdmi_audio_96k() { 0x80 } else { 0 },
        ),
        // Audio Clock Config
        (0x01, 0x00),                                               //
        (0x02, if options.hdmi_audio_96k() { 0x30 } else { 0x18 }), // Set N Value 12288/6144
        (0x03, 0x00),                                               //
        (0x07, 0x01),                                               //
        (0x08, 0x22),                                               // Set CTS Value 74250
        (0x09, 0x0A),                                               //
    ];

    send_to_i2c(&mut i2c, init_data)?;
    hdmi_config_set_csc(&mut i2c, options)?;
    hdmi_config_set_hdr(&mut i2c, options)?;
    Ok(())
}

fn hdmi_config_set_csc(device: &mut LinuxI2CDevice, options: &MisterConfig) -> Result<(), String> {
    // default color conversion matrices
    // for the original hexadecimal versions please refer
    // to the ADV7513 programming guide section 4.3.7

    // no transformation, so use identity matrix
    let hdmi_full_coeffs = Mat4::IDENTITY;

    let hdmi_limited_1_coeffs = matrix! {
        0.8583984375,   0.0,            0.0,            0.06250,
        0.0,            0.8583984375,   0.0,            0.06250,
        0.0,            0.0,            0.8583984375,   0.06250,
        0.0,            0.0,            0.0,            1.0,
    };

    let hdmi_limited_2_coeffs = matrix! {
        0.93701171875,  0.0,            0.0,            0.06250,
        0.0,            0.93701171875,  0.0,            0.06250,
        0.0,            0.0,            0.93701171875,  0.06250,
        0.0,            0.0,            0.0,            1.0,
    };

    let hdr_dcip3_coeffs = matrix! {
        0.8225, 0.1774, 0.0000, 0.0,
        0.0332, 0.9669, 0.0000, 0.0,
        0.0171, 0.0724, 0.9108, 0.0,
        0.0,    0.0,    0.0,    1.0,
    };

    let is_ypbpr = options.vga_mode() == VgaMode::Ypbpr && options.direct_video();
    let hdmi_limited = options.hdmi_limited();

    // Out-of-scope defines, not used with ypbpr
    let mut csc_int16 = [0i16; 16];
    if !is_ypbpr {
        // select the base CSC
        let csc = match options.hdr() {
            HdrConfig::DciP3 => hdr_dcip3_coeffs,
            _ => hdmi_full_coeffs,
        };

        // apply color controls
        let brightness = options.video_brightness();
        let contrast = options.video_contrast();
        let saturation = options.video_saturation();
        let hue = options.video_hue_radian();

        // first apply hue matrix, because it does not touch luminance
        let cos_hue = hue.cos();
        let sin_hue = hue.sin();
        let lr: f32 = 0.213;
        let lg: f32 = 0.715;
        let lb: f32 = 0.072;
        let ca: f32 = 0.143;
        let cb: f32 = 0.140;
        let cc: f32 = 0.283;

        let mat_hue = matrix! {
            lr + cos_hue * (1.0 - lr) + sin_hue * (-lr),
            lg + cos_hue * (-lg) + sin_hue * (-lg),
            lb + cos_hue * (-lb) + sin_hue * (1.0 - lb),
            0.0,
            lr + cos_hue * (-lr) + sin_hue * (ca),
            lg + cos_hue * (1.0 - lg) + sin_hue * (cb),
            lb + cos_hue * (-lb) + sin_hue * (cc),
            0.0,
            lr + cos_hue * (-lr) + sin_hue * (-(1.0 - lr)),
            lg + cos_hue * (-lg) + sin_hue * (lg),
            lb + cos_hue * (1.0 - lb) + sin_hue * (lb),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        };

        let csc = csc * mat_hue;

        // now saturation
        let sr = (1.0f32 - saturation) * 0.3086;
        let sg = (1.0f32 - saturation) * 0.6094;
        let sb = (1.0f32 - saturation) * 0.0920;

        let mat_saturation = matrix! {
            sr + saturation,    0.0,                0.0,                0.0,
            sr,                 sg + saturation,    0.0,                0.0,
            sr,                 sg,                 sb + saturation,    0.0,
            0.0,                0.0,                0.0,                1.0,
        };

        let csc = csc * mat_saturation;

        // now brightness and contrast
        let b = brightness;
        let c = contrast;
        let t = (1.0 - c) / 2.0;

        let mat_brightness_contrast = matrix! {
            c,  0,  0,  (t + b),
            0,  c,  0,  (t + b),
            0,  0,  c,  (t + b),
            0,  0,  0,  1.0,
        };

        let csc = csc * mat_brightness_contrast;

        // gain and offset
        let video::VideoGainOffsets {
            gain_red: rg,
            offset_red: ro,
            gain_green: gg,
            offset_green: go,
            gain_blue: bg,
            offset_blue: bo,
        } = options.video_gain_offset();

        let mat_gain_off = matrix! {
            rg,  0,  0, ro,
             0, gg,  0, go,
             0,  0, bg, bo,
             0,  0,  0,  1
        };

        let mut csc = csc * mat_gain_off;

        // Final compression.
        // Make sure the Matrix is scaled to the range of -2..2.
        // Find the maximum finite value and divide by it.
        let max = csc
            .to_cols_array()
            .into_iter()
            .max_by(|a, b| {
                if a.is_finite() && b.is_finite() {
                    a.partial_cmp(b).unwrap()
                } else if a.is_finite() {
                    std::cmp::Ordering::Less
                } else if b.is_finite() {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .unwrap();
        if max > 2.0 {
            csc = csc * (1.0 / max) * 2.0;
        }

        // Make sure to retain hdmi limited range.
        let csc = match options.hdmi_limited() {
            HdmiLimitedConfig::Limited => csc * hdmi_limited_1_coeffs,
            HdmiLimitedConfig::LimitedForVgaConverters => csc * hdmi_limited_2_coeffs,
            _ => csc,
        };

        // Finally, apply a fixed multiplier to get it in
        // correct range for ADV7513 chip (i16).
        csc_int16
            .iter_mut()
            .zip(csc.to_cols_array().iter())
            .for_each(|(a, b)| {
                *a = (b * 2048.0) as i16;
            });
    };

    // Clamps to reinforce limited if necessary
    // 0x100 = 16/256 * 4096 (12-bit mul)
    // 0xEB0 = 235/256 * 4096
    // 0xFFF = 4095 (12-bit max)
    let clip_min: u16 = if !is_ypbpr && hdmi_limited.is_limited() {
        0x100
    } else {
        0x000
    };
    let clip_max: u16 = if !is_ypbpr && hdmi_limited == HdmiLimitedConfig::LimitedForVgaConverters {
        0xEB0
    } else {
        0xFFF
    };

    // Pass to HDMI, use 0xA0 to set a mode of [-2..2] per ADV7513 programming guide
    #[rustfmt::skip]
        let csc_data: &[(u8, u8)] = &[
        // csc Coefficients, Channel A
        (0x18, if is_ypbpr { 0x86 } else { 0b10100000 | ((csc_int16[0] >> 8) & 0b00011111) as u8 }),
        (0x19, if is_ypbpr { 0xDF } else { (csc_int16[0]) as u8 }),
        (0x1A, if is_ypbpr { 0x1A } else { (csc_int16[1] >> 8) as u8 }),
        (0x1B, if is_ypbpr { 0x3F } else { (csc_int16[1]) as u8 }),
        (0x1C, if is_ypbpr { 0x1E } else { (csc_int16[2] >> 8) as u8 }),
        (0x1D, if is_ypbpr { 0xE2 } else { (csc_int16[2]) as u8 }),
        (0x1E, if is_ypbpr { 0x07 } else { (csc_int16[3] >> 8) as u8 }),
        (0x1F, if is_ypbpr { 0xE7 } else { (csc_int16[3]) as u8 }),

        // csc Coefficients, Channel B
        (0x20, if is_ypbpr { 0x04 } else { (csc_int16[4] >> 8) as u8 }),
        (0x21, if is_ypbpr { 0x1C } else { (csc_int16[4]) as u8 }),
        (0x22, if is_ypbpr { 0x08 } else { (csc_int16[5] >> 8) as u8 }),
        (0x23, if is_ypbpr { 0x11 } else { (csc_int16[5]) as u8 }),
        (0x24, if is_ypbpr { 0x01 } else { (csc_int16[6] >> 8) as u8 }),
        (0x25, if is_ypbpr { 0x91 } else { (csc_int16[6]) as u8 }),
        (0x26, if is_ypbpr { 0x01 } else { (csc_int16[7] >> 8) as u8 }),
        (0x27, if is_ypbpr { 0x00 } else { (csc_int16[7]) as u8 }),

        // csc Coefficients, Channel C
        (0x28, if is_ypbpr { 0x1D } else { (csc_int16[8] >> 8) as u8 }),
        (0x29, if is_ypbpr { 0xAE } else { (csc_int16[8]) as u8 }),
        (0x2A, if is_ypbpr { 0x1B } else { (csc_int16[9] >> 8) as u8 }),
        (0x2B, if is_ypbpr { 0x73 } else { (csc_int16[9]) as u8 }),
        (0x2C, if is_ypbpr { 0x06 } else { (csc_int16[10] >> 8) as u8 }),
        (0x2D, if is_ypbpr { 0xDF } else { (csc_int16[10]) as u8 }),
        (0x2E, if is_ypbpr { 0x07 } else { (csc_int16[11] >> 8) as u8 }),
        (0x2F, if is_ypbpr { 0xE7 } else { (csc_int16[11]) as u8 }),

        // HDMI limited clamps
        (0xC0, (clip_min >> 8) as u8),
        (0xC1, (clip_min) as u8),
        (0xC2, (clip_max >> 8) as u8),
        (0xC3, (clip_max) as u8),
    ];

    send_to_i2c(device, csc_data)?;
    Ok(())
}

fn hdmi_config_set_hdr(device: &mut LinuxI2CDevice, options: &MisterConfig) -> Result<(), String> {
    // Grab desired nits values
    let max_nits = options.hdr_max_nits();
    let avg_nits = options.hdr_avg_nits();

    // CTA-861-G: 6.9 Dynamic Range and Mastering InfoFrame
    // Uses BT2020 RGB primaries and white point chromacity
    // Max Lum: 1000cd/m2, Min Lum: 0cd/m2, MaxCLL: 1000cd/m2
    // MaxFALL: 250cd/m2 (this value does not matter much -
    // in essence it means that the display should expect -
    // 25% of the image to be 1000cd/m2)
    // If HDR == 1, use HLG
    #[rustfmt::skip]
        let hdr_data: &mut [u8] = &mut [
        0x87,
        0x01,
        0x1a,
        0x00, // Checksum, calculate later
        (if options.hdr() == HdrConfig::Hlg { 0x03 } else { 0x02 }),
        0x48,
        0x8a,
        0x08,
        0x39,
        0x34,
        0x21,
        0xaa,
        0x9b,
        0x96,
        0x19,
        0xfc,
        0x08,
        0x13,
        0x3d,
        0x42,
        0x40,
        0x00,
        max_nits as u8,
        (max_nits >> 8) as u8,
        0x01,
        0x00,
        max_nits as u8,
        (max_nits >> 8) as u8,
        avg_nits as u8,
        (avg_nits >> 8) as u8,
    ];

    // now we calculate the checksum for this packet (2s complement sum)
    hdr_data[3] = !(hdr_data.iter().copied().fold(0u8, |a, i| a.wrapping_add(i))) + 1;

    fn hdmi_config_set_spare(i2c: &mut LinuxI2CDevice, enabled: bool) -> Result<(), String> {
        let mask = 0x02;

        let mut packet_val = i2c.smbus_read_byte_data(0x40).map_err(|error| {
            error!(?error, "i2c: read error (0x40)");
            error.to_string()
        })?;

        if enabled {
            packet_val |= mask;
        } else {
            packet_val &= !mask;
        }

        i2c.smbus_write_byte_data(0x40, packet_val)
            .map_err(|error| {
                error!(?error, "i2c: write error (0x40, {:02X})", packet_val);
                error.to_string()
            })
    }

    if options.hdr() == HdrConfig::None {
        hdmi_config_set_spare(device, false)?;
        return Ok(());
    }

    hdmi_config_set_spare(device, true)?;
    let mut i2c = create_i2c(0x38)?;
    if let Err(error) = i2c.smbus_write_byte_data(0xFF, 0b10000000) {
        error!(
            ?error,
            "i2c: hdr: Couldn't update Spare Packet change register (0xDF, 0x80)"
        );
    }

    hdr_data
        .into_iter()
        .enumerate()
        .map(|(i, val)| ((i + 0xE0) as u8, *val))
        .map(|(addr, val)| {
            i2c.smbus_write_byte_data(addr, val).map_err(|error| {
                error!(
                    ?error,
                    "i2c: hdr register write error (0x{:02X}, 0x{:02X})", addr, val
                );
                error.to_string()
            })
        })
        .collect::<Result<(), String>>()?;

    i2c.smbus_write_byte_data(0xfF, 0x00).map_err(|error| {
        error!(
            ?error,
            "i2c: hdr: Couldn't update Spare Packet change register (0xDF, 0x00)"
        );
        error.to_string()
    })
}

pub fn init_mode(
    options: &config::MisterConfig,
    core: &mut crate::core::MisterFpgaCore,
    is_menu: bool,
) -> Result<(), String> {
    video_mode::init_mode(options, core.spi_mut(), is_menu)
}

pub fn select_mode(
    mode: CustomVideoMode,
    direct_video: bool,
    aspect_ratio_1: Option<AspectRatio>,
    aspect_ratio_2: Option<AspectRatio>,
    spi: &mut Spi<impl MemoryMapper>,
    is_menu: bool,
) -> Result<(), String> {
    video_mode::select_mode(
        mode,
        Default::default(),
        0,
        direct_video,
        aspect_ratio_1,
        aspect_ratio_2,
        spi,
        is_menu,
    )
}
