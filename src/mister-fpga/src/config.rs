use merge::Merge;
use num_traits::FloatConst;
use serde::Deserialize;
use serde_with::{serde_as, DeserializeFromStr, DurationSeconds};
use std::collections::HashMap;
use std::ffi::{c_char, c_int};
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use thiserror::Error;
use tracing::info;
use validator::Validate;
use video::aspect::AspectRatio;
use video::resolution::Resolution;

mod bootcore;
mod fb_size;
mod hdmi_limited;
mod hdr;
mod ini; // Internal module.
mod ntsc_mode;
mod osd_rotate;
mod reset_combo;
mod vga_mode;
pub mod video;
mod vrr_mode;
mod vscale_mode;
mod vsync_adjust;

pub use bootcore::*;
pub use fb_size::*;
pub use hdmi_limited::*;
pub use hdr::*;
pub use ntsc_mode::*;
pub use osd_rotate::*;
pub use reset_combo::*;
pub use vga_mode::*;
pub use video::*;
pub use vrr_mode::*;
pub use vscale_mode::*;
pub use vsync_adjust::*;

mod cpp {
    use libc::*;

    #[repr(C)]
    pub struct CppCfg {
        pub keyrah_mode: u32,
        pub forced_scandoubler: u8,
        pub key_menu_as_rgui: u8,
        pub reset_combo: u8,
        pub csync: u8,
        pub vga_scaler: u8,
        pub vga_sog: u8,
        pub hdmi_audio_96k: u8,
        pub dvi_mode: u8,
        pub hdmi_limited: u8,
        pub direct_video: u8,
        pub video_info: u8,
        pub refresh_min: c_float,
        pub refresh_max: c_float,
        pub controller_info: u8,
        pub vsync_adjust: u8,
        pub kbd_nomouse: u8,
        pub mouse_throttle: u8,
        pub bootscreen: u8,
        pub vscale_mode: u8,
        pub vscale_border: u16,
        pub rbf_hide_datecode: u8,
        pub menu_pal: u8,
        pub bootcore_timeout: i16,
        pub fb_size: u8,
        pub fb_terminal: u8,
        pub osd_rotate: u8,
        pub osd_timeout: u16,
        pub gamepad_defaults: u8,
        pub recents: u8,
        pub jamma_vid: u16,
        pub jamma_pid: u16,
        pub no_merge_vid: u16,
        pub no_merge_pid: u16,
        pub no_merge_vidpid: [u32; 256],
        pub spinner_vid: u16,
        pub spinner_pid: u16,
        pub spinner_throttle: c_int,
        pub spinner_axis: u8,
        pub sniper_mode: u8,
        pub browse_expand: u8,
        pub logo: u8,
        pub log_file_entry: u8,
        pub shmask_mode_default: u8,
        pub bt_auto_disconnect: c_int,
        pub bt_reset_before_pair: c_int,
        pub bootcore: [c_char; 256],
        pub video_conf: [c_char; 1024],
        pub video_conf_pal: [c_char; 1024],
        pub video_conf_ntsc: [c_char; 1024],
        pub font: [c_char; 1024],
        pub shared_folder: [c_char; 1024],
        pub waitmount: [c_char; 1024],
        pub custom_aspect_ratio: [[c_char; 16]; 2],
        pub afilter_default: [c_char; 1023],
        pub vfilter_default: [c_char; 1023],
        pub vfilter_vertical_default: [c_char; 1023],
        pub vfilter_scanlines_default: [c_char; 1023],
        pub shmask_default: [c_char; 1023],
        pub preset_default: [c_char; 1023],
        pub player_controller: [[[c_char; 256]; 8]; 6],
        pub rumble: u8,
        pub wheel_force: u8,
        pub wheel_range: u16,
        pub hdmi_game_mode: u8,
        pub vrr_mode: u8,
        pub vrr_min_framerate: u8,
        pub vrr_max_framerate: u8,
        pub vrr_vesa_framerate: u8,
        pub video_off: u16,
        pub disable_autofire: u8,
        pub video_brightness: u8,
        pub video_contrast: u8,
        pub video_saturation: u8,
        pub video_hue: u16,
        pub video_gain_offset: [c_char; 256],
        pub hdr: u8,
        pub hdr_max_nits: u16,
        pub hdr_avg_nits: u16,
        pub vga_mode: [c_char; 16],
        pub vga_mode_int: c_char,
        pub ntsc_mode: c_char,
        pub controller_unique_mapping: [u32; 256],
    }
}

#[derive(Error, Debug)]
#[error(transparent)]
pub enum ConfigError {
    #[error("Invalid config file: {0}")]
    Io(#[from] io::Error),

    #[error("Could not read INI file: {0}")]
    IniError(#[from] ini::Error),

    #[error("Could not read JSON file: {0}")]
    JsonError(#[from] json5::Error),
}

/// A helper function to read minutes into durations from the config.
struct DurationMinutes;

impl<'de> serde_with::DeserializeAs<'de, Duration> for DurationMinutes {
    fn deserialize_as<D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let dur: Duration = DurationSeconds::<u64>::deserialize_as(deserializer)?;
        let secs = dur.as_secs();
        Ok(Duration::from_secs(secs * 60))
    }
}

/// A struct representing the `video=123x456` string for sections in the config.
#[derive(DeserializeFromStr, Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct VideoResolutionString(Resolution);

impl FromStr for VideoResolutionString {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_prefix("video=") {
            Ok(VideoResolutionString(Resolution::from_str(s)?))
        } else {
            Err("Invalid video section string.")
        }
    }
}

/// Serde helper for supporting mister booleans (can be 0 or 1) used in the current config.
mod mister_bool {
    use serde::{Deserialize, Deserializer};

    /// Transforms a mister string into a boolean.
    fn bool_from_str(s: impl AsRef<str>) -> bool {
        match s.as_ref() {
            "enabled" | "true" | "1" => true,
            "0" => false,
            _ => false,
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::deserialize(deserializer).map(|v: Option<String>| v.map(bool_from_str))
    }
}

/// Serde helper for supporting mister hexa values (can be 0x1234) used in the current config.
mod mister_hexa {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D, T: num_traits::Num>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::deserialize(deserializer).and_then(|v: Option<String>| {
            if let Some(s) = v {
                if let Some(hs) = s.strip_prefix("0x") {
                    T::from_str_radix(hs, 16)
                        .map_err(|_| {
                            serde::de::Error::custom(format!("Invalid hexadecimal value: {s}"))
                        })
                        .map(Some)
                } else if s == "0" {
                    Ok(Some(T::zero()))
                } else {
                    Err(serde::de::Error::custom(format!(
                        "Invalid hexadecimal value: {}",
                        s
                    )))
                }
            } else {
                Ok(None)
            }
        })
    }
}

/// Serde helper for supporting mister hexa values (can be 0x1234) used in the current config.
mod mister_hexa_seq {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D, T: num_traits::Num>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::deserialize(deserializer).and_then(|v: Vec<String>| {
            v.into_iter()
                .map(|s| {
                    if let Some(hs) = s.strip_prefix("0x") {
                        T::from_str_radix(hs, 16).map_err(|_| {
                            serde::de::Error::custom(format!("Invalid hexadecimal value: {s}"))
                        })
                    } else if s == "0" {
                        Ok(T::zero())
                    } else {
                        Err(serde::de::Error::custom(format!(
                            "Invalid hexadecimal value: {}",
                            s
                        )))
                    }
                })
                .collect()
        })
    }
}

mod validate {
    use std::time::Duration;
    use validator::ValidationError;

    pub fn video_info(video_info: &Duration) -> Result<(), ValidationError> {
        if video_info.as_secs() > 10 || video_info.as_secs() < 1 {
            return Err(ValidationError::new("video_info must be between 1 and 10."));
        }

        Ok(())
    }

    pub fn controller_info(controller_info: &Duration) -> Result<(), ValidationError> {
        if controller_info.as_secs() > 10 {
            return Err(ValidationError::new(
                "controller_info must be between 0 and 10.",
            ));
        }

        Ok(())
    }

    pub fn osd_timeout(osd_timeout: &Duration) -> Result<(), ValidationError> {
        if osd_timeout.as_secs() > 3600 || osd_timeout.as_secs() < 5 {
            return Err(ValidationError::new(
                "osd_timeout must be between 5 and 3600.",
            ));
        }

        Ok(())
    }

    pub fn bootcore_timeout(bootcore_timeout: &Duration) -> Result<(), ValidationError> {
        if bootcore_timeout.as_secs() > 30 {
            return Err(ValidationError::new("bootcore_timeout must be 30 or less."));
        }

        Ok(())
    }

    pub fn video_off(video_off: &Duration) -> Result<(), ValidationError> {
        if video_off.as_secs() > 3600 {
            return Err(ValidationError::new("video_off must be 3600 or less."));
        }

        Ok(())
    }
}

/// The `[MiSTer]` section of the configuration file. This represents a single MiSTer section,
/// which contains most configurations.
///
/// The `Mister.ini` file specifies that one can override the default values by specifying the
/// `[MiSTer]` section with resolution specific configurations, e.g. `[video=320x240]`.
/// Because of this, we pretty much set everything as optional and defines the default in the
/// getters on the [`Config`] struct.
///
/// This allows us to overwrite only options which are defined in the subsections.
#[serde_as]
#[derive(Default, Debug, Clone, Deserialize, Merge, Validate)]
#[serde(default)]
pub struct MisterConfig {
    #[merge(strategy = merge::option::overwrite_some)]
    pub bootcore: Option<BootCoreConfig>,

    #[serde(alias = "ypbpr")]
    #[merge(strategy = merge::option::overwrite_some)]
    pub vga_mode: Option<VgaMode>,

    #[merge(strategy = merge::option::overwrite_some)]
    pub ntsc_mode: Option<NtscModeConfig>,

    #[merge(strategy = merge::option::overwrite_some)]
    pub reset_combo: Option<ResetComboConfig>,

    #[merge(strategy = merge::option::overwrite_some)]
    pub hdmi_limited: Option<HdmiLimitedConfig>,

    #[merge(strategy = merge::option::overwrite_some)]
    #[validate(range(max = 100))]
    pub mouse_throttle: Option<u8>,

    #[serde(with = "mister_hexa")]
    #[merge(strategy = merge::option::overwrite_some)]
    pub keyrah_mode: Option<u32>,

    /// Specify a custom aspect ratio in the format `a:b`. This can be repeated.
    /// They are applied in order, so the first one matching will be the one used.
    #[merge(strategy = merge::vec::append)]
    custom_aspect_ratio: Vec<AspectRatio>,

    /// Specify a custom aspect ratio, allowing for backward compatibility with older
    /// MiSTer config files. We only need 2 as that's what the previous version supported.
    #[merge(strategy = merge::option::overwrite_some)]
    pub custom_aspect_ratio_1: Option<AspectRatio>,

    /// Specify a custom aspect ratio, allowing for backward compatibility with older
    /// MiSTer config files. We only need 2 as that's what the previous version supported.
    #[merge(strategy = merge::option::overwrite_some)]
    pub custom_aspect_ratio_2: Option<AspectRatio>,

    /// Set to 1 to run scandoubler on VGA output always (depends on core).
    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    pub forced_scandoubler: Option<bool>,

    /// Set to true to make the MENU key map to RGUI in Minimig (e.g. for Right Amiga).
    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    pub key_menu_as_rgui: Option<bool>,

    /// Set to true for composite sync on HSync signal of VGA output.
    #[serde(with = "mister_bool", alias = "csync")]
    #[merge(strategy = merge::option::overwrite_some)]
    pub composite_sync: Option<bool>,

    /// Set to true to connect VGA to scaler output.
    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    pub vga_scaler: Option<bool>,

    /// Set to true to enable sync on green (needs analog I/O board v6.0 or newer).
    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    pub vga_sog: Option<bool>,

    /// Set to true for 96khz/16bit HDMI audio (48khz/16bit otherwise)
    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    pub hdmi_audio_96k: Option<bool>,

    /// Set to true for DVI mode. Audio won't be transmitted through HDMI in DVI mode.
    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    pub dvi_mode: Option<bool>,

    /// Set to true to enable core video timing over HDMI, use only with VGA converters.
    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    pub direct_video: Option<bool>,

    /// Set to 0-10 (seconds) to display video info on startup/change
    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    #[merge(strategy = merge::option::overwrite_some)]
    #[validate(custom(function = validate::video_info))]
    pub video_info: Option<Duration>,

    /// 1-10 (seconds) to display controller's button map upon first time key press
    /// 0 - disable
    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    #[validate(custom(function = validate::controller_info))]
    #[merge(strategy = merge::option::overwrite_some)]
    pub controller_info: Option<Duration>,

    /// If you monitor doesn't support either very low (NTSC monitors may not support PAL) or
    /// very high (PAL monitors may not support NTSC) then you can set refresh_min and/or refresh_max
    /// parameters, so vsync_adjust won't be applied for refreshes outside specified.
    /// These parameters are valid only when vsync_adjust is non-zero.
    #[validate(range(min = 0.0, max = 150.0))]
    #[merge(strategy = merge::option::overwrite_some)]
    pub refresh_min: Option<f32>,

    /// If you monitor doesn't support either very low (NTSC monitors may not support PAL) or
    /// very high (PAL monitors may not support NTSC) then you can set refresh_min and/or refresh_max
    /// parameters, so vsync_adjust won't be applied for refreshes outside specified.
    /// These parameters are valid only when vsync_adjust is non-zero.
    #[validate(range(min = 0.0, max = 150.0))]
    #[merge(strategy = merge::option::overwrite_some)]
    refresh_max: Option<f32>,

    /// Set to 1 for automatic HDMI VSync rate adjust to match original VSync.
    /// Set to 2 for low latency mode (single buffer).
    /// This option makes video butter smooth like on original emulated system.
    /// Adjusting is done by changing pixel clock. Not every display supports variable pixel clock.
    /// For proper adjusting and to reduce possible out of range pixel clock, use 60Hz HDMI video
    /// modes as a base even for 50Hz systems.
    #[merge(strategy = merge::option::overwrite_some)]
    vsync_adjust: Option<VsyncAdjustConfig>,

    // TODO: figure this out.
    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    kbd_nomouse: Option<bool>,

    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    bootscreen: Option<bool>,

    /// 0 - scale to fit the screen height.
    /// 1 - use integer scale only.
    /// 2 - use 0.5 steps of scale.
    /// 3 - use 0.25 steps of scale.
    /// 4 - integer resolution scaling, use core aspect ratio
    /// 5 - integer resolution scaling, maintain display aspect ratio
    #[merge(strategy = merge::option::overwrite_some)]
    vscale_mode: Option<VideoScaleModeConfig>,

    /// Set vertical border for TVs cutting the upper/bottom parts of screen (1-399)
    #[validate(range(min = 0, max = 399))]
    #[merge(strategy = merge::option::overwrite_some)]
    pub vscale_border: Option<u16>,

    /// true - hides datecodes from rbf file names. Press F2 for quick temporary toggle
    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    rbf_hide_datecode: Option<bool>,

    /// 1 - PAL mode for menu core
    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    menu_pal: Option<bool>,

    /// 10-30 timeout before autoboot, comment for autoboot without timeout.
    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    #[validate(custom(function = validate::bootcore_timeout))]
    #[merge(strategy = merge::option::overwrite_some)]
    bootcore_timeout: Option<Duration>,

    /// 0 - automatic, 1 - full size, 2 - 1/2 of resolution, 4 - 1/4 of resolution.
    #[merge(strategy = merge::option::overwrite_some)]
    pub fb_size: Option<FramebufferSizeConfig>,

    /// TODO: figure this out.
    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    fb_terminal: Option<bool>,

    /// Display OSD menu rotated,  0 - no rotation, 1 - rotate right (+90°), 2 - rotate left (-90°)
    #[merge(strategy = merge::option::overwrite_some)]
    osd_rotate: Option<OsdRotateConfig>,

    /// 5-3600 timeout (in seconds) for OSD to disappear in Menu core. 0 - never timeout.
    /// Background picture will get darker after double timeout.
    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    #[validate(custom(function = validate::osd_timeout))]
    #[merge(strategy = merge::option::overwrite_some)]
    osd_timeout: Option<Duration>,

    /// Defines internal joypad mapping from virtual SNES mapping in main to core mapping
    /// Set to 0 for name mapping (jn) (e.g. A button in SNES core = A button on controller regardless of position on pad)
    /// Set to 1 for positional mapping (jp) (e.g. A button in SNES core = East button on controller regardless of button name)
    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    gamepad_defaults: Option<bool>,

    /// 1 - enables the recent file loaded/mounted.
    /// WARNING: This option will enable write to SD card on every load/mount which may wear the SD card after many writes to the same place
    ///          There is also higher chance to corrupt the File System if MiSTer will be reset or powered off while writing.
    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    recents: Option<bool>,

    /// JammaSD/J-PAC/I-PAC keys to joysticks translation
    /// You have to provide correct VID and PID of your input device
    /// Examples: Legacy J-PAC with Mini-USB or USB capable I-PAC with PS/2 connectors VID=0xD209/PID=0x0301
    /// USB Capable J-PAC with only PS/2 connectors VID=0x04B4/PID=0x0101
    /// JammaSD: VID=0x04D8/PID=0xF3AD
    #[serde(with = "mister_hexa")]
    #[merge(strategy = merge::option::overwrite_some)]
    jamma_vid: Option<u16>,

    #[serde(with = "mister_hexa")]
    #[merge(strategy = merge::option::overwrite_some)]
    jamma_pid: Option<u16>,

    /// Disable merging input devices. Use if only Player1 works.
    /// Leave no_merge_pid empty to apply this to all devices with the same VID.
    #[serde(with = "mister_hexa")]
    #[merge(strategy = merge::option::overwrite_some)]
    no_merge_vid: Option<u16>,

    #[serde(with = "mister_hexa")]
    #[merge(strategy = merge::option::overwrite_some)]
    no_merge_pid: Option<u16>,

    #[serde(with = "mister_hexa_seq")]
    #[merge(strategy = merge::vec::append)]
    no_merge_vidpid: Vec<u32>,

    /// use specific (VID/PID) mouse X movement as a spinner and paddle. Use VID=0xFFFF/PID=0xFFFF to use all mice as spinners.
    #[serde(with = "mister_hexa")]
    #[merge(strategy = merge::option::overwrite_some)]
    spinner_vid: Option<u16>,

    #[serde(with = "mister_hexa")]
    #[merge(strategy = merge::option::overwrite_some)]
    spinner_pid: Option<u16>,

    // I WAS HERE.
    #[validate(range(min = -10000, max = 10000))]
    #[merge(strategy = merge::option::overwrite_some)]
    spinner_throttle: Option<i32>,

    #[merge(strategy = merge::option::overwrite_some)]
    spinner_axis: Option<u8>,

    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    sniper_mode: Option<bool>,

    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    browse_expand: Option<bool>,

    /// 0 - disable MiSTer logo in Menu core
    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    logo: Option<bool>,

    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    log_file_entry: Option<bool>,

    #[merge(strategy = merge::option::overwrite_some)]
    shmask_mode_default: Option<u8>,

    /// Automatically disconnect (and shutdown) Bluetooth input device if not use specified amount of time.
    /// Some controllers have no automatic shutdown built in and will keep connection till battery dry out.
    /// 0 - don't disconnect automatically, otherwise it's amount of minutes.
    #[serde_as(as = "Option<DurationMinutes>")]
    #[merge(strategy = merge::option::overwrite_some)]
    bt_auto_disconnect: Option<Duration>,

    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    bt_reset_before_pair: Option<bool>,

    #[serde(alias = "video_mode")]
    #[merge(strategy = merge::option::overwrite_some)]
    video_conf: Option<String>,

    #[serde(alias = "video_mode_pal")]
    #[merge(strategy = merge::option::overwrite_some)]
    video_conf_pal: Option<String>,

    #[serde(alias = "video_mode_ntsc")]
    #[merge(strategy = merge::option::overwrite_some)]
    video_conf_ntsc: Option<String>,

    #[merge(strategy = merge::option::overwrite_some)]
    font: Option<String>,

    #[merge(strategy = merge::option::overwrite_some)]
    shared_folder: Option<String>,

    #[merge(strategy = merge::option::overwrite_some)]
    waitmount: Option<String>,

    #[merge(strategy = merge::option::overwrite_some)]
    afilter_default: Option<String>,

    #[merge(strategy = merge::option::overwrite_some)]
    vfilter_default: Option<String>,

    #[merge(strategy = merge::option::overwrite_some)]
    vfilter_vertical_default: Option<String>,

    #[merge(strategy = merge::option::overwrite_some)]
    vfilter_scanlines_default: Option<String>,

    #[merge(strategy = merge::option::overwrite_some)]
    shmask_default: Option<String>,

    #[merge(strategy = merge::option::overwrite_some)]
    preset_default: Option<String>,

    #[serde(default)]
    #[merge(strategy = merge::vec::append)]
    player_controller: Vec<Vec<String>>,

    #[serde(default)]
    #[merge(strategy = merge::vec::append)]
    player_1_controller: Vec<String>,
    #[serde(default)]
    #[merge(strategy = merge::vec::append)]
    player_2_controller: Vec<String>,
    #[serde(default)]
    #[merge(strategy = merge::vec::append)]
    player_3_controller: Vec<String>,
    #[serde(default)]
    #[merge(strategy = merge::vec::append)]
    player_4_controller: Vec<String>,
    #[serde(default)]
    #[merge(strategy = merge::vec::append)]
    player_5_controller: Vec<String>,
    #[serde(default)]
    #[merge(strategy = merge::vec::append)]
    player_6_controller: Vec<String>,

    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    rumble: Option<bool>,

    #[merge(strategy = merge::option::overwrite_some)]
    #[validate(range(min = 0, max = 100))]
    wheel_force: Option<u8>,

    #[merge(strategy = merge::option::overwrite_some)]
    #[validate(range(min = 0, max = 1000))]
    wheel_range: Option<u16>,

    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    hdmi_game_mode: Option<bool>,

    /// Variable Refresh Rate control
    /// 0 - Do not enable VRR (send no VRR control frames)
    /// 1 - Auto Detect VRR from display EDID.
    /// 2 - Force Enable Freesync
    /// 3 - Force Enable Vesa HDMI Forum VRR
    #[merge(strategy = merge::option::overwrite_some)]
    vrr_mode: Option<VrrModeConfig>,

    #[merge(strategy = merge::option::overwrite_some)]
    vrr_min_framerate: Option<u8>,

    #[merge(strategy = merge::option::overwrite_some)]
    vrr_max_framerate: Option<u8>,

    #[merge(strategy = merge::option::overwrite_some)]
    vrr_vesa_framerate: Option<u8>,

    /// output black frame in Menu core after timeout (is seconds). Valid only if osd_timout is non-zero.
    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    #[merge(strategy = merge::option::overwrite_some)]
    #[validate(custom(function = validate::video_off))]
    video_off: Option<Duration>,

    #[serde(with = "mister_bool")]
    #[merge(strategy = merge::option::overwrite_some)]
    disable_autofire: Option<bool>,

    #[merge(strategy = merge::option::overwrite_some)]
    #[validate(range(min = 0, max = 100))]
    video_brightness: Option<u8>,

    #[merge(strategy = merge::option::overwrite_some)]
    #[validate(range(min = 0, max = 100))]
    video_contrast: Option<u8>,

    #[merge(strategy = merge::option::overwrite_some)]
    #[validate(range(min = 0, max = 100))]
    video_saturation: Option<u8>,

    #[merge(strategy = merge::option::overwrite_some)]
    #[validate(range(min = 0, max = 360))]
    video_hue: Option<u16>,

    #[merge(strategy = merge::option::overwrite_some)]
    video_gain_offset: Option<VideoGainOffsets>,

    #[merge(strategy = merge::option::overwrite_some)]
    hdr: Option<HdrConfig>,

    #[merge(strategy = merge::option::overwrite_some)]
    #[validate(range(min = 100, max = 10000))]
    hdr_max_nits: Option<u16>,

    #[merge(strategy = merge::option::overwrite_some)]
    #[validate(range(min = 100, max = 10000))]
    hdr_avg_nits: Option<u16>,

    #[serde(with = "mister_hexa_seq")]
    #[merge(strategy = merge::vec::append)]
    controller_unique_mapping: Vec<u32>,
}

impl MisterConfig {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_defaults() -> Self {
        let mut config = Self::new();
        config.set_defaults();
        config
    }

    /// Set the default values for this config.
    pub fn set_defaults(&mut self) {
        self.bootscreen.get_or_insert(true);
        self.fb_terminal.get_or_insert(true);
        self.controller_info.get_or_insert(Duration::from_secs(6));
        self.browse_expand.get_or_insert(true);
        self.logo.get_or_insert(true);
        self.rumble.get_or_insert(true);
        self.wheel_force.get_or_insert(50);
        self.hdr.get_or_insert(HdrConfig::default());
        self.hdr_avg_nits.get_or_insert(250);
        self.hdr_max_nits.get_or_insert(1000);
        self.video_brightness.get_or_insert(50);
        self.video_contrast.get_or_insert(50);
        self.video_saturation.get_or_insert(100);
        self.video_hue.get_or_insert(0);
        self.video_gain_offset
            .get_or_insert("1, 0, 1, 0, 1, 0".parse().unwrap());
        self.video_conf.get_or_insert("6".to_string());
    }

    pub fn custom_aspect_ratio(&self) -> Vec<AspectRatio> {
        if self.custom_aspect_ratio.is_empty() {
            self.custom_aspect_ratio_1
                .iter()
                .chain(self.custom_aspect_ratio_2.iter())
                .copied()
                .collect()
        } else {
            self.custom_aspect_ratio.clone()
        }
    }

    pub fn player_controller(&self) -> Vec<Vec<String>> {
        if self.player_controller.is_empty() {
            let mut vec = Vec::new();
            if !self.player_1_controller.is_empty() {
                vec.push(self.player_1_controller.clone());
            }
            if !self.player_2_controller.is_empty() {
                vec.push(self.player_2_controller.clone());
            }
            if !self.player_3_controller.is_empty() {
                vec.push(self.player_3_controller.clone());
            }
            if !self.player_4_controller.is_empty() {
                vec.push(self.player_4_controller.clone());
            }
            if !self.player_5_controller.is_empty() {
                vec.push(self.player_5_controller.clone());
            }
            if !self.player_6_controller.is_empty() {
                vec.push(self.player_6_controller.clone());
            }

            vec
        } else {
            self.player_controller.clone()
        }
    }

    #[inline]
    pub fn hdmi_limited(&self) -> HdmiLimitedConfig {
        self.hdmi_limited.unwrap_or_default()
    }

    #[inline]
    pub fn hdmi_game_mode(&self) -> bool {
        self.hdmi_game_mode.unwrap_or_default()
    }

    #[inline]
    pub fn hdr(&self) -> HdrConfig {
        self.hdr.unwrap_or_default()
    }

    #[inline]
    pub fn hdr_max_nits(&self) -> u16 {
        self.hdr_max_nits.unwrap_or(1000)
    }

    #[inline]
    pub fn hdr_avg_nits(&self) -> u16 {
        self.hdr_avg_nits.unwrap_or(250)
    }

    #[inline]
    pub fn dvi_mode(&self) -> bool {
        self.dvi_mode.unwrap_or_default()
    }

    #[inline]
    pub fn dvi_mode_raw(&self) -> Option<bool> {
        self.dvi_mode
    }

    #[inline]
    pub fn hdmi_audio_96k(&self) -> bool {
        self.hdmi_audio_96k.unwrap_or_default()
    }

    /// The video brightness, between [-0.5..0.5]
    #[inline]
    pub fn video_brightness(&self) -> f32 {
        (self.video_brightness.unwrap_or(50).clamp(0, 100) as f32 / 100.0) - 0.5
    }

    /// The video contrast, between [0..2]
    #[inline]
    pub fn video_contrast(&self) -> f32 {
        ((self.video_contrast.unwrap_or(50).clamp(0, 100) as f32 / 100.0) - 0.5) * 2. + 1.
    }

    /// The video saturation, between [0..1]
    #[inline]
    pub fn video_saturation(&self) -> f32 {
        self.video_saturation.unwrap_or(100).clamp(0, 100) as f32 / 100.
    }

    /// The video hue.
    #[inline]
    pub fn video_hue_radian(&self) -> f32 {
        (self.video_hue.unwrap_or_default() as f32) * f32::PI() / 180.
    }

    /// The video gains and offets.
    #[inline]
    pub fn video_gain_offset(&self) -> VideoGainOffsets {
        self.video_gain_offset.unwrap_or_default()
    }

    /// The VGA mode.
    #[inline]
    pub fn vga_mode(&self) -> VgaMode {
        self.vga_mode.unwrap_or_default()
    }

    /// Direct Video?
    #[inline]
    pub fn direct_video(&self) -> bool {
        self.direct_video.unwrap_or_default()
    }

    /// Whether to use vsync adjust.
    #[inline]
    pub fn vsync_adjust(&self) -> VsyncAdjustConfig {
        if self.direct_video() {
            VsyncAdjustConfig::Disabled
        } else {
            self.vsync_adjust.unwrap_or_default()
        }
    }

    /// Whether to use PAL in the menu.
    #[inline]
    pub fn menu_pal(&self) -> bool {
        self.menu_pal.unwrap_or_default()
    }

    /// Whether to force the scan doubler.
    #[inline]
    pub fn forced_scandoubler(&self) -> bool {
        self.forced_scandoubler.unwrap_or_default()
    }
}

#[cfg(test)]
pub mod testing {
    use std::path::PathBuf;

    pub(super) static mut ROOT: Option<PathBuf> = None;

    #[cfg(test)]
    pub fn set_config_root(root: impl Into<PathBuf>) {
        unsafe {
            ROOT = Some(root.into());
        }
    }
}

#[derive(Default, Debug, Clone, Deserialize, Merge)]
#[serde(default)]
pub struct Config {
    /// The MiSTer section which should be 99% of config files.
    #[serde(rename = "MiSTer")]
    mister: MisterConfig,

    /// The `[video=123x456@78]` sections, or core section.
    #[serde(flatten)]
    #[merge(strategy = merge::hashmap::recurse)]
    overrides: HashMap<String, MisterConfig>,
}

impl Config {
    fn root() -> PathBuf {
        #[cfg(test)]
        {
            unsafe { testing::ROOT.clone().unwrap() }
        }

        #[cfg(not(test))]
        PathBuf::from("/media/fat")
    }

    pub fn into_inner(self) -> MisterConfig {
        self.mister
    }

    pub fn into_inner_with_overrides(self, overrides: &[&str]) -> MisterConfig {
        let mut mister = self.mister;
        for o in overrides {
            if let Some(override_config) = self.overrides.get(&o.to_string()) {
                mister.merge(override_config.clone());
            }
        }
        mister
    }

    pub fn base() -> Self {
        let path = Self::root().join("MiSTer.ini");
        Self::load(&path).unwrap_or_else(|_| {
            info!(?path, "Failed to load MiSTer.ini, using defaults.");
            let mut c = Self::default();
            c.mister.set_defaults();
            c
        })
    }

    pub fn cores_root() -> PathBuf {
        Self::root()
    }

    pub fn config_root() -> PathBuf {
        Self::root().join("config")
    }

    pub fn last_core_data() -> Option<String> {
        std::fs::read_to_string(Self::config_root().join("lastcore.dat")).ok()
    }

    pub fn merge_core_override(&mut self, corename: &str) {
        if let Some(o) = self.overrides.get(corename) {
            self.mister.merge(o.clone());
        }
    }

    pub fn merge_video_override(&mut self, resolution: Resolution) {
        // Try to get `123x456@78` format first.
        let video_str = format!("video={}", resolution);
        if let Some(o) = self.overrides.get(&video_str) {
            self.mister.merge(o.clone());
        }
    }

    /// Read INI config using our custom parser, then output the JSON, then parse that into
    /// the config struct. This is surprisingly fast, solid and byte compatible with the
    /// original CPP code (checked manually on various files).
    pub fn from_ini<R: io::Read>(mut content: R) -> Result<Self, ConfigError> {
        let mut s = String::new();
        content.read_to_string(&mut s)?;

        if s.is_empty() {
            return Ok(Default::default());
        }

        let json = ini::parse(&s).unwrap().to_json_string(
            |name, value: &str| match name {
                "mouse_throttle"
                | "video_info"
                | "controller_info"
                | "refresh_min"
                | "refresh_max"
                | "vscale_border"
                | "bootcore_timeout"
                | "osd_timeout"
                | "spinner_throttle"
                | "spinner_axis"
                | "shmask_mode_default"
                | "bt_auto_disconnect"
                | "wheel_force"
                | "wheel_range"
                | "vrr_min_framerate"
                | "vrr_max_framerate"
                | "vrr_vesa_framerate"
                | "video_off"
                | "video_brightness"
                | "video_contrast"
                | "video_saturation"
                | "video_hue"
                | "hdr_max_nits"
                | "hdr_avg_nits" => Some(value.to_string()),
                _ => None,
            },
            |name| {
                [
                    "custom_aspect_ratio",
                    "no_merge_vidpid",
                    "player_controller",
                    "player_1_controller",
                    "player_2_controller",
                    "player_3_controller",
                    "player_4_controller",
                    "player_5_controller",
                    "player_6_controller",
                    "controller_unique_mapping",
                ]
                .contains(&name)
            },
            |name: &str| -> Option<&str> {
                if name == "ypbpr" {
                    Some("vga_mode")
                } else {
                    None
                }
            },
        );

        Config::from_json(json.as_bytes())
    }

    pub fn from_json<R: io::Read>(mut content: R) -> Result<Self, ConfigError> {
        let mut c = String::new();
        content.read_to_string(&mut c)?;
        Ok(json5::from_str(&c)?)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("ini") => Self::from_ini(std::fs::File::open(path)?),
            Some("json") => Self::from_json(std::fs::File::open(path)?),
            _ => Err(
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid config file extension").into(),
            ),
        }
    }

    /// Copy this configuration to the C++ side.
    ///
    /// # Safety
    /// This function is unsafe because it writes to a C++ struct.
    pub unsafe fn copy_to_cfg_cpp(self, dest: &mut cpp::CppCfg) {
        unsafe fn copy_string<const N: usize>(dest: &mut [c_char; N], src: &str) {
            let src = src.as_bytes();
            let l = src.len().min(N);
            for i in 0..l {
                let c = src[i] as c_char;
                dest[i] = c;
            }
            dest[l] = 0;
        }

        let m = &self.mister;

        dest.keyrah_mode = m.keyrah_mode.unwrap_or_default();
        dest.forced_scandoubler = m.forced_scandoubler.unwrap_or_default() as u8;
        dest.key_menu_as_rgui = m.key_menu_as_rgui.unwrap_or_default() as u8;
        dest.reset_combo = m.reset_combo.unwrap_or_default() as u8;
        dest.csync = m.composite_sync.unwrap_or_default() as u8;
        dest.vga_scaler = m.vga_scaler.unwrap_or_default() as u8;
        dest.vga_sog = m.vga_sog.unwrap_or_default() as u8;
        dest.hdmi_audio_96k = m.hdmi_audio_96k.unwrap_or_default() as u8;
        // For some reason, dvi_mode is set to 2 by default in the C++ code, instead of false.
        dest.dvi_mode = m.dvi_mode.map(|x| x as u8).unwrap_or(2);
        dest.hdmi_limited = m.hdmi_limited.unwrap_or_default() as u8;
        dest.direct_video = m.direct_video.unwrap_or_default() as u8;
        dest.video_info = m.video_info.unwrap_or_default().as_secs() as u8;
        dest.refresh_min = m.refresh_min.unwrap_or_default();
        dest.refresh_max = m.refresh_max.unwrap_or_default();
        dest.controller_info = m.controller_info.unwrap_or_default().as_secs() as u8;
        dest.vsync_adjust = m.vsync_adjust.unwrap_or_default() as u8;
        dest.kbd_nomouse = m.kbd_nomouse.unwrap_or_default() as u8;
        dest.mouse_throttle = m.mouse_throttle.unwrap_or_default();
        dest.bootscreen = m.bootscreen.unwrap_or_default() as u8;
        dest.vscale_mode = m.vscale_mode.unwrap_or_default() as u8;
        dest.vscale_border = m.vscale_border.unwrap_or_default();
        dest.rbf_hide_datecode = m.rbf_hide_datecode.unwrap_or_default() as u8;
        dest.menu_pal = m.menu_pal.unwrap_or_default() as u8;
        dest.bootcore_timeout = m.bootcore_timeout.unwrap_or_default().as_secs() as i16;
        dest.fb_size = m.fb_size.unwrap_or_default() as u8;
        dest.fb_terminal = m.fb_terminal.unwrap_or_default() as u8;
        dest.osd_rotate = m.osd_rotate.unwrap_or_default() as u8;
        dest.osd_timeout = m.osd_timeout.unwrap_or_default().as_secs() as u16;
        dest.gamepad_defaults = m.gamepad_defaults.unwrap_or_default() as u8;
        dest.recents = m.recents.unwrap_or_default() as u8;
        dest.jamma_vid = m.jamma_vid.unwrap_or_default();
        dest.jamma_pid = m.jamma_pid.unwrap_or_default();
        dest.no_merge_vid = m.no_merge_vid.unwrap_or_default();
        dest.no_merge_pid = m.no_merge_pid.unwrap_or_default();
        dest.no_merge_vidpid = [0; 256];
        for (i, vidpid) in m.no_merge_vidpid.iter().enumerate() {
            dest.no_merge_vidpid[i] = *vidpid;
        }
        dest.spinner_vid = m.spinner_vid.unwrap_or_default();
        dest.spinner_pid = m.spinner_pid.unwrap_or_default();
        dest.spinner_throttle = m.spinner_throttle.unwrap_or_default() as c_int;
        dest.spinner_axis = m.spinner_axis.unwrap_or_default();
        dest.sniper_mode = m.sniper_mode.unwrap_or_default() as u8;
        dest.browse_expand = m.browse_expand.unwrap_or_default() as u8;
        dest.logo = m.logo.unwrap_or_default() as u8;
        dest.log_file_entry = m.log_file_entry.unwrap_or_default() as u8;
        dest.shmask_mode_default = m.shmask_mode_default.unwrap_or_default();
        dest.bt_auto_disconnect =
            (m.bt_auto_disconnect.unwrap_or_default().as_secs() / 60) as c_int;
        dest.bt_reset_before_pair = m.bt_reset_before_pair.unwrap_or_default() as c_int;
        copy_string(
            &mut dest.bootcore,
            &m.bootcore.clone().unwrap_or_default().to_string(),
        );
        copy_string(
            &mut dest.video_conf,
            &m.video_conf.clone().unwrap_or_default(),
        );
        copy_string(
            &mut dest.video_conf_pal,
            &m.video_conf_pal.clone().unwrap_or_default(),
        );
        copy_string(
            &mut dest.video_conf_ntsc,
            &m.video_conf_ntsc.clone().unwrap_or_default(),
        );
        copy_string(&mut dest.font, &m.font.clone().unwrap_or_default());
        copy_string(
            &mut dest.shared_folder,
            &m.shared_folder.clone().unwrap_or_default(),
        );
        copy_string(
            &mut dest.waitmount,
            &m.waitmount.clone().unwrap_or_default(),
        );
        let ar = m.custom_aspect_ratio();
        let aspect_ratio_1 = ar.first();
        let aspect_ratio_2 = ar.get(1);
        if let Some(a1) = aspect_ratio_1 {
            copy_string(&mut dest.custom_aspect_ratio[0], &a1.to_string());
        }
        if let Some(a2) = aspect_ratio_2 {
            copy_string(&mut dest.custom_aspect_ratio[1], &a2.to_string());
        }
        copy_string(
            &mut dest.afilter_default,
            &m.afilter_default.clone().unwrap_or_default(),
        );
        copy_string(
            &mut dest.vfilter_default,
            &m.vfilter_default.clone().unwrap_or_default(),
        );
        copy_string(
            &mut dest.vfilter_vertical_default,
            &m.vfilter_vertical_default.clone().unwrap_or_default(),
        );
        copy_string(
            &mut dest.vfilter_scanlines_default,
            &m.vfilter_scanlines_default.clone().unwrap_or_default(),
        );
        copy_string(
            &mut dest.shmask_default,
            &m.shmask_default.clone().unwrap_or_default(),
        );
        copy_string(
            &mut dest.preset_default,
            &m.preset_default.clone().unwrap_or_default(),
        );
        let player_controller = m.player_controller();
        for (i, pc) in player_controller.iter().enumerate().take(6) {
            for (j, p) in pc.iter().enumerate() {
                copy_string(&mut dest.player_controller[i][j], p);
            }
        }
        dest.rumble = m.rumble.unwrap_or_default() as u8;
        dest.wheel_force = m.wheel_force.unwrap_or_default();
        dest.wheel_range = m.wheel_range.unwrap_or_default();
        dest.hdmi_game_mode = m.hdmi_game_mode.unwrap_or_default() as u8;
        dest.vrr_mode = m.vrr_mode.unwrap_or_default() as u8;
        dest.vrr_min_framerate = m.vrr_min_framerate.unwrap_or_default();
        dest.vrr_max_framerate = m.vrr_max_framerate.unwrap_or_default();
        dest.vrr_vesa_framerate = m.vrr_vesa_framerate.unwrap_or_default();
        dest.video_off = m.video_off.unwrap_or_default().as_secs() as u16;
        dest.disable_autofire = m.disable_autofire.unwrap_or_default() as u8;
        dest.video_brightness = m.video_brightness.unwrap_or_default();
        dest.video_contrast = m.video_contrast.unwrap_or_default();
        dest.video_saturation = m.video_saturation.unwrap_or_default();
        dest.video_hue = m.video_hue.unwrap_or_default();
        copy_string(
            &mut dest.video_gain_offset,
            &match m.video_gain_offset.as_ref() {
                Some(v) => v.to_string(),
                None => "1, 0, 1, 0, 1, 0".to_string(),
            },
        );
        dest.hdr = m.hdr.unwrap_or_default() as u8;
        dest.hdr_max_nits = m.hdr_max_nits.unwrap_or_default();
        dest.hdr_avg_nits = m.hdr_avg_nits.unwrap_or_default();
        copy_string(
            &mut dest.vga_mode,
            &m.vga_mode.unwrap_or_default().to_string(),
        );
        dest.vga_mode_int = m.vga_mode.unwrap_or_default() as u8 as c_char;
        dest.ntsc_mode = m.ntsc_mode.unwrap_or_default() as u8 as c_char;
        for (i, mapping) in m.controller_unique_mapping.iter().enumerate().take(256) {
            dest.controller_unique_mapping[i] = *mapping;
        }
    }

    /// Merge a configuration file with another.
    pub fn merge(&mut self, other: Config) {
        Merge::merge(self, other);
    }
}

#[test]
fn works_with_empty_file() {
    unsafe {
        let mut cpp_cfg: cpp::CppCfg = std::mem::zeroed();
        let config = Config::from_ini(io::empty()).unwrap();
        config.copy_to_cfg_cpp(&mut cpp_cfg);
    }
}

#[cfg(test)]
mod examples {
    use crate::config::*;

    #[rstest::rstest]
    fn works_with_example(#[files("tests/assets/config/*.ini")] p: PathBuf) {
        unsafe {
            let mut cpp_cfg: cpp::CppCfg = std::mem::zeroed();
            let config = Config::load(p).unwrap();
            config.copy_to_cfg_cpp(&mut cpp_cfg);
        }
    }
}
