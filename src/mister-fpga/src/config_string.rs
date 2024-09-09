//! Parses the config string of a core, and extract the proper parameters.
//! See this documentation for more information:
//! https://github.com/MiSTer-devel/MkDocs_MiSTer/blob/main/docs/developer/conf_str.md
//!
//! This is located in utils to allow to run test. There is no FPGA or MiSTer specific
//! code in this module, even though it isn't used outside of MiSTer itself.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, Range};
use std::path::Path;
use std::str::FromStr;

use once_cell::sync::Lazy;
use regex::Regex;
use tracing::{debug, warn};

use one_fpga::core::{CoreSettingItem, CoreSettings, SettingId};
pub use types::*;

use crate::fpga::user_io;
use crate::types::StatusBitMap;

pub mod midi;
pub mod settings;
pub mod uart;

mod parser;

mod types;

static LABELED_SPEED_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d*)(\([^)]*\))?").unwrap());

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FileExtension(pub [u8; 3]);

impl Debug for FileExtension {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FileExtension")
            .field(&format_args!("'{}'", self.as_str()))
            .finish()
    }
}

impl Deref for FileExtension {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { std::str::from_utf8_unchecked(&self.0).trim() }
    }
}

impl FromStr for FileExtension {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > 3 {
            return Err("Invalid file extension");
        }

        let bytes = s.as_bytes();
        Ok(Self([
            bytes.first().copied().unwrap_or(b' '),
            bytes.get(1).copied().unwrap_or(b' '),
            bytes.get(2).copied().unwrap_or(b' '),
        ]))
    }
}

impl FileExtension {
    pub fn as_str(&self) -> &str {
        self.deref()
    }
}

#[derive(Debug, Clone)]
pub struct LoadFileInfo {
    /// Core supports save files, load a file, and mount a save for reading or writing
    pub save_support: bool,

    /// Explicit index (or index is generated from line number if index not given).
    pub index: u8,

    /// A concatenated list of 3 character file extensions. For example, BINGEN would be
    /// BIN and GEN extensions.
    pub extensions: Vec<FileExtension>,

    /// Optional {Text} string is the text that is displayed before the extensions like
    /// "Load RAM". If {Text} is not specified, then default is "Load *".
    pub label: Option<String>,

    /// Optional {Address} - load file directly into DDRAM at this address.
    /// ioctl_index from hps_io will be: ioctl_index[5:0] = index(explicit or auto),
    /// ioctl_index[7:6] = extension index
    pub address: Option<FpgaRamMemoryAddress>,
}

impl LoadFileInfo {
    pub fn setting_id(&self) -> SettingId {
        self.label.as_ref().map_or_else(
            || SettingId::new(self.index as u32),
            |l| SettingId::from_label(&l),
        )
    }
}

/// A component of a Core config string.
#[derive(Debug, Clone)]
pub enum ConfigMenu {
    /// Empty lines, potentially with text.
    Empty(Option<String>),

    /// Cheat menu option.
    Cheat(Option<String>),

    /// Disable the option if the menu mask is set.
    DisableIf(u32, Box<ConfigMenu>),

    /// Disable the option if the menu mask is NOT set.
    DisableUnless(u32, Box<ConfigMenu>),

    /// Hide the option if the menu mask is set.
    HideIf(u32, Box<ConfigMenu>),

    /// Hide the option if the menu mask is NOT set.
    HideUnless(u32, Box<ConfigMenu>),

    /// DIP switch menu option.
    Dip,

    /// Load file menu option.
    LoadFile(Box<LoadFileInfo>),

    /// Open file and remember it, useful for remembering an alternative rom, config, or other
    /// type of file. See [Self::LoadFile] for more information.
    LoadFileAndRemember(Box<LoadFileInfo>),

    /// Mount SD card menu option.
    MountSdCard {
        slot: u8,
        extensions: Vec<FileExtension>,
        label: Option<String>,
    },

    /// `O{Index1}[{Index2}],{Name},{Options...}` - Option button that allows you to select
    /// between various choices.
    ///
    /// - `{Index1}` and `{Index2}` are values from 0-9 and A-V (like Hex, but it extends
    ///   from A-V instead of A-F). This represents all 31 bits. First and second index
    ///   are the range of bits that will be set in the status register.
    /// - `{Name}` is what is shown to describe the option.
    /// - {Options...} - a list of comma separated options.
    Option {
        bits: Range<u8>,
        label: String,
        choices: Vec<String>,
    },

    /// `T{Index},{Name}` - Trigger button. This is a simple button that will pulse HIGH of
    /// specified `{Index}` bit in status register. A perfect example of this is for a reset
    /// button. `{Name}` is the text that describes the button function.
    /// `R{Index},{Name}` is the same but should close the OSD.
    Trigger {
        close_osd: bool,
        index: u8,
        label: String,
    },

    /// `J[1],{Button1}[,{Button2},...]` - J1 means lock keyboard to joystick emulation mode.
    /// Useful for keyboard-less systems such as consoles. `{Button1},{Button2},...` is list
    /// of joystick buttons used in the core. Up to 12 buttons can be listed. Analog axis
    /// are not defined here. The user just needs to map them through the Menu core.
    JoystickButtons {
        /// If true, the keyboard should be locked to joystick emulation mode.
        keyboard: bool,

        /// List of buttons that can be mapped.
        buttons: Vec<String>,
    },

    SnesButtonDefaultList {
        buttons: Vec<String>,
    },

    SnesButtonDefaultPositionalList {
        buttons: Vec<String>,
    },

    /// `P{#},{Title}` - Creates sub-page for options with `{Title}`.
    /// `P{#}` - Prefix to place the option into specific `{#}` page. This is added
    ///          before O# but after something like d#. (e.g.
    ///          "d5P1o2,Vertical Crop,Disabled,216p(5x);", is correct and
    ///          "P1d5o2,Vertical Crop,Disabled,216p(5x);", is incorrect and the menu
    ///          options will not work).
    Page {
        /// The page number.
        index: u8,

        /// The title of the page.
        label: String,
    },

    /// A page item, which is a sub-menu. The first item is the page
    /// index, the second is the item.
    PageItem(u8, Box<ConfigMenu>),

    /// `I,INFO1,INFO2,...,INFO255` - INFO1-INFO255 lines to display as OSD info (top left
    /// corner of screen).
    Info(Vec<String>),

    /// `V,{Version String}` - Version string. {Version String} is the version string.
    /// Takes the core name and appends version string for name to display.
    Version(String),
}

impl ConfigMenu {
    pub fn as_option(&self) -> Option<&Self> {
        match self {
            ConfigMenu::Option { .. } => Some(self),
            ConfigMenu::DisableIf(_, sub)
            | ConfigMenu::DisableUnless(_, sub)
            | ConfigMenu::HideIf(_, sub)
            | ConfigMenu::HideUnless(_, sub)
            | ConfigMenu::PageItem(_, sub) => sub.as_option(),
            _ => None,
        }
    }

    pub fn as_trigger(&self) -> Option<&Self> {
        match self {
            ConfigMenu::Trigger { .. } => Some(self),
            ConfigMenu::DisableIf(_, sub)
            | ConfigMenu::DisableUnless(_, sub)
            | ConfigMenu::HideIf(_, sub)
            | ConfigMenu::HideUnless(_, sub)
            | ConfigMenu::PageItem(_, sub) => sub.as_trigger(),
            _ => None,
        }
    }

    pub fn as_core_menu_item(&self, status: &StatusBitMap) -> Vec<CoreSettingItem> {
        match self {
            ConfigMenu::LoadFile(info) | ConfigMenu::LoadFileAndRemember(info) => {
                vec![CoreSettingItem::file_select(
                    info.setting_id(),
                    info.label
                        .as_ref()
                        .map_or_else(|| "Load File", String::as_str),
                    info.extensions.iter().map(|e| e.to_string()).collect(),
                )]
            }
            ConfigMenu::Option {
                label,
                choices,
                bits,
            } => {
                let value = status.get_range(bits.clone());
                match choices.len() {
                    0 => vec![],
                    1 => vec![CoreSettingItem::trigger(label, label)],
                    2 => vec![CoreSettingItem::bool_option(label, label, Some(value != 0))],
                    _ => vec![CoreSettingItem::int_option(
                        label,
                        label,
                        choices.clone(),
                        Some(value as usize % choices.len()),
                    )],
                }
            }
            ConfigMenu::Trigger { label, .. } => vec![CoreSettingItem::trigger(label, label)],
            ConfigMenu::Page { label, .. } => {
                vec![CoreSettingItem::page(label, label, label, Vec::new())]
            }
            ConfigMenu::PageItem(_, sub) => sub.as_core_menu_item(status),
            ConfigMenu::HideIf(mask, sub) => {
                if status.get(*mask as usize) {
                    vec![]
                } else {
                    sub.as_core_menu_item(status)
                }
            }
            ConfigMenu::HideUnless(mask, sub) => {
                if !status.get(*mask as usize) {
                    vec![]
                } else {
                    sub.as_core_menu_item(status)
                }
            }
            ConfigMenu::DisableIf(mask, sub) => sub
                .as_core_menu_item(status)
                .into_iter()
                .map(|item| item.with_disabled(status.get(*mask as usize)))
                .collect(),
            ConfigMenu::DisableUnless(mask, sub) => sub
                .as_core_menu_item(status)
                .into_iter()
                .map(|item| item.with_disabled(!status.get(*mask as usize)))
                .collect(),
            ConfigMenu::Empty(label) => {
                if let Some(label) = label {
                    vec![CoreSettingItem::label(false, label)]
                } else {
                    vec![CoreSettingItem::Separator]
                }
            }
            ConfigMenu::Info(_) => vec![],
            ConfigMenu::Version(v) => {
                vec![CoreSettingItem::label(false, &format!("Version: {}", v))]
            }
            _ => vec![],
        }
    }

    pub fn as_load_file(&self) -> Option<&Self> {
        match self {
            ConfigMenu::LoadFile(_) | ConfigMenu::LoadFileAndRemember(_) => Some(self),
            ConfigMenu::DisableIf(_, sub)
            | ConfigMenu::DisableUnless(_, sub)
            | ConfigMenu::HideIf(_, sub)
            | ConfigMenu::HideUnless(_, sub)
            | ConfigMenu::PageItem(_, sub) => sub.as_load_file(),
            _ => None,
        }
    }

    pub fn as_load_file_info(&self) -> Option<&LoadFileInfo> {
        match self {
            ConfigMenu::LoadFile(info) | ConfigMenu::LoadFileAndRemember(info) => Some(info),
            _ => None,
        }
    }

    pub fn setting_id(&self) -> Option<SettingId> {
        match self {
            ConfigMenu::Page { label, .. } => Some(SettingId::from_label(&label)),
            ConfigMenu::Option { label, .. } => Some(SettingId::from_label(&label)),
            ConfigMenu::Trigger { label, .. } => Some(SettingId::from_label(&label)),
            ConfigMenu::PageItem(_, sub) => sub.setting_id(),
            ConfigMenu::HideIf(_, sub)
            | ConfigMenu::DisableIf(_, sub)
            | ConfigMenu::HideUnless(_, sub)
            | ConfigMenu::DisableUnless(_, sub) => sub.setting_id(),
            _ => None,
        }
    }

    pub fn label(&self) -> Option<&str> {
        match self {
            ConfigMenu::DisableIf(_, sub)
            | ConfigMenu::DisableUnless(_, sub)
            | ConfigMenu::HideIf(_, sub)
            | ConfigMenu::HideUnless(_, sub) => sub.label(),
            // TODO: add those.
            // ConfigMenu::Cheat(name) => name.as_ref().map(|x| x.as_str()),
            ConfigMenu::LoadFileAndRemember(info) | ConfigMenu::LoadFile(info) => {
                info.label.as_ref().map(|l| l.as_str())
            }
            ConfigMenu::Option { label, .. } => Some(label.as_str()),
            ConfigMenu::Trigger { label, .. } => Some(label.as_str()),
            ConfigMenu::PageItem(_, sub) => sub.label(),
            _ => None,
        }
    }

    pub fn page(&self) -> Option<u8> {
        match self {
            ConfigMenu::DisableIf(_, inner) => inner.page(),
            ConfigMenu::DisableUnless(_, inner) => inner.page(),
            ConfigMenu::HideIf(_, inner) => inner.page(),
            ConfigMenu::HideUnless(_, inner) => inner.page(),
            ConfigMenu::PageItem(index, _) => Some(*index),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    /// The name of the core.
    pub name: String,

    /// Unordered set of settings, e.g. save state memory and UART speed.
    pub settings: settings::Settings,

    /// Menu of the core.
    pub menu: Vec<ConfigMenu>,
}

impl Config {
    /// Create a new config from the FPGA.
    /// This is disabled in Test as this module is still included in the test build.
    pub fn from_fpga(fpga: &mut crate::fpga::MisterFpga) -> Result<Self, String> {
        let mut cfg_string = String::with_capacity(1024);
        fpga.spi_mut()
            .execute(user_io::UserIoGetString(&mut cfg_string))?;
        debug!(?cfg_string, "Config string from FPGA");

        Self::from_str(&cfg_string)
    }

    pub fn settings(&self) -> &settings::Settings {
        &self.settings
    }

    pub fn status_bit_map_mask(&self) -> StatusBitMap {
        let mut arr = StatusBitMap::new();
        // First bit is always 1 for soft reset (reserved).
        arr.set(0, true);

        for item in self.menu.iter() {
            if let Some(ConfigMenu::Option { ref bits, .. }) = item.as_option() {
                for i in bits.clone() {
                    arr.set(i as usize, true);
                }
            } else if let Some(ConfigMenu::Trigger { index, .. }) = item.as_trigger() {
                arr.set(*index as usize, true);
            }
        }

        arr
    }

    pub fn load_info(&self, path: impl AsRef<Path>) -> Result<Option<LoadFileInfo>, String> {
        let path_ext = match path.as_ref().extension() {
            Some(ext) => ext.to_string_lossy(),
            None => return Err("No extension".to_string()),
        };

        for item in self.menu.iter() {
            if let ConfigMenu::LoadFile(ref info) = item {
                if info
                    .extensions
                    .iter()
                    .any(|ext| ext.eq_ignore_ascii_case(&path_ext))
                {
                    return Ok(Some(info.as_ref().clone()));
                }
            }
        }
        Ok(None)
    }

    pub fn snes_default_button_list(&self) -> Option<&Vec<String>> {
        for item in self.menu.iter() {
            if let ConfigMenu::SnesButtonDefaultList { ref buttons } = item {
                return Some(buttons);
            }
        }
        None
    }

    pub fn version(&self) -> Option<&str> {
        for item in self.menu.iter() {
            if let ConfigMenu::Version(ref version) = item {
                return Some(version);
            }
        }
        None
    }

    pub fn as_core_settings(&self, bits: &StatusBitMap) -> CoreSettings {
        let it = self.menu.iter().flat_map(|item| {
            item.as_core_menu_item(bits)
                .into_iter()
                .map(move |i| (item, i))
        });

        let mut root = Vec::new();
        let mut pages: HashMap<u8, usize> = HashMap::new();
        for (config_menu, core_menu) in it {
            if let ConfigMenu::Page { index, .. } = config_menu {
                pages.insert(*index, root.len());
                root.push(core_menu);
                continue;
            }

            // Find the page number.
            match config_menu.page() {
                None | Some(0) => root.push(core_menu),
                Some(page) => {
                    if let Some(i) = pages.get(&page) {
                        let page = root.get_mut(*i).unwrap();
                        page.add_item(core_menu);
                    } else {
                        warn!(?page, "Page hasn't been created yet");
                    }
                }
            }
        }

        CoreSettings::new(self.name.clone(), root)
    }
}

impl FromStr for Config {
    type Err = String;

    fn from_str(cfg_string: &str) -> Result<Self, Self::Err> {
        let (rest, (name, settings, menu)) =
            parser::parse_config_menu(cfg_string.into()).map_err(|e| e.to_string())?;

        if !rest.fragment().is_empty() {
            return Err(format!(
                "Did not parse config string to the end. Rest: '{}'",
                rest.fragment()
            ));
        }

        Ok(Self {
            name,
            settings,
            menu,
        })
    }
}

// Taken from https://github.com/MiSTer-devel/NES_MiSTer/blob/7a645f4/NES.sv#L219
#[cfg(test)]
const CONFIG_STRING_NES: &str = "\
        NES;SS3E000000:200000,UART31250,MIDI;\
        FS,NESFDSNSF;\
        H1F2,BIN,Load FDS BIOS;\
        -;\
        ONO,System Type,NTSC,PAL,Dendy;\
        -;\
        C,Cheats;\
        H2OK,Cheats Enabled,On,Off;\
        -;\
        oI,Autosave,On,Off;\
        H5D0R6,Load Backup RAM;\
        H5D0R7,Save Backup RAM;\
        -;\
        oC,Savestates to SDCard,On,Off;\
        oDE,Savestate Slot,1,2,3,4;\
        d7rA,Save state(Alt+F1-F4);\
        d7rB,Restore state(F1-F4);\
        -;\
        P1,Audio & Video;\
        P1-;\
        P1oFH,Palette,Kitrinx,Smooth,Wavebeam,Sony CXA,PC-10 Better,Custom;\
        H3P1FC3,PAL,Custom Palette;\
        P1-;\
        P1OIJ,Aspect ratio,Original,Full Screen,[ARC1],[ARC2];\
        P1O13,Scandoubler Fx,None,HQ2x,CRT 25%,CRT 50%,CRT 75%;\
        d6P1O5,Vertical Crop,Disabled,216p(5x);\
        d6P1o36,Crop Offset,0,2,4,8,10,12,-12,-10,-8,-6,-4,-2;\
        P1o78,Scale,Normal,V-Integer,Narrower HV-Integer,Wider HV-Integer;\
        P1-;\
        P1O4,Hide Overscan,Off,On;\
        P1ORS,Mask Edges,Off,Left,Both,Auto;\
        P1OP,Extra Sprites,Off,On;\
        P1-;\
        P1OUV,Audio Enable,Both,Internal,Cart Expansion,None;\
        P2,Input Options;\
        P2-;\
        P2O9,Swap Joysticks,No,Yes;\
        P2OA,Multitap,Disabled,Enabled;\
        P2oJK,SNAC,Off,Controllers,Zapper,3D Glasses;\
        P2o02,Periphery,None,Zapper(Mouse),Zapper(Joy1),Zapper(Joy2),Vaus,Vaus(A-Trigger),Powerpad,Family Trainer;\
        P2oL,Famicom Keyboard,No,Yes;\
        P2-;\
        P2OL,Zapper Trigger,Mouse,Joystick;\
        P2OM,Crosshairs,On,Off;\
        P3,Miscellaneous;\
        P3-;\
        P3OG,Disk Swap,Auto,FDS button;\
        P3o9,Pause when OSD is open,Off,On;\
        - ;\
        R0,Reset;\
        J1,A,B,Select,Start,FDS,Mic,Zapper/Vaus Btn,PP/Mat 1,PP/Mat 2,PP/Mat 3,PP/Mat 4,PP/Mat 5,PP/Mat 6,PP/Mat 7,PP/Mat 8,PP/Mat 9,PP/Mat 10,PP/Mat 11,PP/Mat 12,Savestates;\
        jn,A,B,Select,Start,L,,R|P;\
        jp,B,Y,Select,Start,L,,R|P;\
        I,\
        Disk 1A,\
        Disk 1B,\
        Disk 2A,\
        Disk 2B,\
        Slot=DPAD|Save/Load=Start+DPAD,\
        Active Slot 1,\
        Active Slot 2,\
        Active Slot 3,\
        Active Slot 4,\
        Save to state 1,\
        Restore state 1,\
        Save to state 2,\
        Restore state 2,\
        Save to state 3,\
        Restore state 3,\
        Save to state 4,\
        Restore state 4;\
        V,v123456";

#[test]
fn config_string_nes() {
    let config = Config::from_str(CONFIG_STRING_NES);
    assert!(config.is_ok(), "{:?}", config);
}

#[test]
fn config_string_nes_menu() {
    let config = Config::from_str(CONFIG_STRING_NES).unwrap();
    config.as_core_settings(&StatusBitMap::new());
}

#[test]
fn config_string_chess() {
    // Taken from https://github.com/MiSTer-devel/Chess_MiSTer/blob/113b6f6/Chess.sv#L182
    let config = Config::from_str(
        [
            "Chess;;",
            "-;",
            "O7,Opponent,AI,Human;",
            "O46,AI Strength,1,2,3,4,5,6,7;",
            "O23,AI Randomness,0,1,2,3;",
            "O1,Player Color,White,Black;",
            "O9,Boardview,White,Black;",
            "OA,Overlay,Off,On;",
            "-;",
            "O8,Aspect Ratio,4:3,16:9;",
            "-;",
            "R0,Reset;",
            "J1,Action,Cancel,SaveState,LoadState,Rewind;",
            "jn,A,B;",
            "jp,A,B;",
            "V,v221106",
        ]
        .join("")
        .as_str(),
    );

    assert!(config.is_ok(), "{:?}", config);
    let config = config.unwrap();
    assert!(config.settings.uart_mode.is_empty());

    // From running the core on MiSTer:
    //
    // Status Bit Map:
    //              Upper                          Lower
    // 0         1         2         3          4         5         6
    // 01234567890123456789012345678901 23456789012345678901234567890123
    // 0123456789ABCDEFGHIJKLMNOPQRSTUV 0123456789ABCDEFGHIJKLMNOPQRSTUV
    // XXXXXXXXXXX

    let map = config.status_bit_map_mask();
    let data = map.as_raw_slice();
    let expected = [
        0b00000111_11111111u16,
        0b00000000_00000000,
        0b00000000_00000000,
        0b00000000_00000000,
        0b00000000_00000000,
        0b00000000_00000000,
        0b00000000_00000000,
        0b00000000_00000000,
    ];
    assert_eq!(
        data, expected,
        "actual: {:016b}{:016b}{:016b}{:016b}\nexpect: {:016b}{:016b}{:016b}{:016b}",
        data[0], data[1], data[2], data[3], expected[0], expected[1], expected[2], expected[3]
    );
}

#[test]
fn config_string_ao486() {
    // From https://github.com/MiSTer-devel/ao486_MiSTer/blob/09b29b2/ao486.sv#L199
    let config = Config::from_str(
        [
            "AO486;UART115200:4000000(Turbo 115200),MIDI;",
            "S0,IMGIMAVFD,Floppy A:;",
            "S1,IMGIMAVFD,Floppy B:;",
            "O12,Write Protect,None,A:,B:,A: & B:;",
            "-;",
            "S2,VHD,IDE 0-0;",
            "S3,VHD,IDE 0-1;",
            "-;",
            "S4,VHDISOCUECHD,IDE 1-0;",
            "S5,VHDISOCUECHD,IDE 1-1;",
            "-;",
            "oJM,CPU Preset,User Defined,~PC XT-7MHz,~PC AT-8MHz,~PC AT-10MHz,~PC AT-20MHz,~PS/2-20MHz,~386SX-25MHz,~386DX-33Mhz,~386DX-40Mhz,~486SX-33Mhz,~486DX-33Mhz,MAX (unstable);",
            "-;",
            "P1,Audio & Video;",
            "P1-;",
            "P1OMN,Aspect ratio,Original,Full Screen,[ARC1],[ARC2];",
            "P1O4,VSync,60Hz,Variable;",
            "P1O8,16/24bit mode,BGR,RGB;",
            "P1O9,16bit format,1555,565;",
            "P1OE,Low-Res,Native,4x;", "P1oDE,Scale,Normal,V-Integer,Narrower HV-Integer,Wider HV-Integer;", "P1-;", "P1O3,FM mode,OPL2,OPL3;", "P1OH,C/MS,Disable,Enable;", "P1OIJ,Speaker Volume,1,2,3,4;", "P1OKL,Audio Boost,No,2x,4x;", "P1oBC,Stereo Mix,none,25%,50%,100%;", "P1OP,MT32 Volume Ctl,MIDI,Line-In;", "P2,Hardware;", "P2o01,Boot 1st,Floppy/Hard Disk,Floppy,Hard Disk,CD-ROM;", "P2o23,Boot 2nd,NONE,Floppy,Hard Disk,CD-ROM;", "P2o45,Boot 3rd,NONE,Floppy,Hard Disk,CD-ROM;",
            "P2-;",
            "P2o6,IDE 1-0 CD Hot-Swap,Yes,No;",
            "P2o7,IDE 1-1 CD Hot-Swap,No,Yes;",
            "P2-;",
            "P2OB,RAM Size,256MB,16MB;",
            "P2-;",
            "P2OA,USER I/O,MIDI,COM2;",
            "P2-;",
            "P2OCD,Joystick type,2 Buttons,4 Buttons,Gravis Pro,None;",
            "P2oFG,Joystick Mode,2 Joysticks,2 Sticks,2 Wheels,4-axes Wheel;",
            "P2oH,Joystick 1,Enabled,Disabled;",
            "P2oI,Joystick 2,Enabled,Disabled;",
            "h3P3,MT32-pi;",
            "h3P3-;",
            "h3P3OO,Use MT32-pi,Yes,No;",
            "h3P3o9A,Show Info,No,Yes,LCD-On(non-FB),LCD-Auto(non-FB);",
            "h3P3-;",
            "h3P3-,Default Config:;",
            "h3P3OQ,Synth,Munt,FluidSynth;",
            "h3P3ORS,Munt ROM,MT-32 v1,MT-32 v2,CM-32L;",
            "h3P3OTV,SoundFont,0,1,2,3,4,5,6,7;",
            "h3P3-;",
            "h3P3r8,Reset Hanging Notes;",
            "-;",
            "R0,Reset and apply HDD;",
            "J,Button 1,Button 2,Button 3,Button 4,Start,Select,R1,L1,R2,L2;",
            "jn,A,B,X,Y,Start,Select,R,L;",
            "I,",
            "MT32-pi: SoundFont #0,",
            "MT32-pi: SoundFont #1,",
            "MT32-pi: SoundFont #2,",
            "MT32-pi: SoundFont #3,",
            "MT32-pi: SoundFont #4,",
            "MT32-pi: SoundFont #5,",
            "MT32-pi: SoundFont #6,",
            "MT32-pi: SoundFont #7,",
            "MT32-pi: MT-32 v1,",
            "MT32-pi: MT-32 v2,",
            "MT32-pi: CM-32L,",
            "MT32-pi: Unknown mode;",
            "V,v123456"].join("").as_str()
    );

    assert!(config.is_ok(), "{:?}", config);
}

#[test]
fn input_tester() {
    let config = Config::from_str(
        "InputTest;;-;\
        O35,Scandoubler Fx,None,HQ2x,CRT 25%,CRT 50%,CRT 75%;\
        OGJ,Analog Video H-Pos,0,-1,-2,-3,-4,-5,-6,-7,8,7,6,5,4,3,2,1;\
        OKN,Analog Video V-Pos,0,-1,-2,-3,-4,-5,-6,-7,8,7,6,5,4,3,2,1;\
        O89,Aspect ratio,Original,Full Screen,[ARC1],[ARC2];\
        -;\
        O6,Rotate video,Off,On;\
        O7,Flip video,Off,On;\
        -;\
        RA,Open menu;\
        -;\
        F0,BIN,Load BIOS;\
        F3,BIN,Load Sprite ROM;\
        F4,YM,Load Music (YM5/6);\
        -;\
        R0,Reset;\
        J,A,B,X,Y,L,R,Select,Start;\
        V,v220825",
    );

    assert!(config.is_ok(), "{:?}", config);
    let config = config.unwrap();
    assert!(config.settings.uart_mode.is_empty());
}

#[test]
fn config_string_gba() {
    let config = Config::from_str(
        "GBA;SS3E000000:80000;\
        FS,GBA,Load,300C0000;\
        -;\
        C,Cheats;\
        H1O[6],Cheats Enabled,Yes,No;\
        -;\
        D0R[12],Reload Backup RAM;\
        D0R[13],Save Backup RAM;\
        D0O[23],Autosave,Off,On;\
        D0-;\
        O[36],Savestates to SDCard,On,Off;\
        O[43],Autoincrement Slot,Off,On;\
        O[38:37],Savestate Slot,1,2,3,4;\
        h4H3R[17],Save state (Alt-F1);\
        h4H3R[18],Restore state (F1);\
        -;\
        P1,Video & Audio;\
        P1-;\
        P1O[33:32],Aspect ratio,Original,Full Screen,[ARC1],[ARC2];\
        P1O[4:2],Scandoubler Fx,None,HQ2x,CRT 25%,CRT 50%,CRT 75%;\
        P1O[35:34],Scale,Normal,V-Integer,Narrower HV-Integer,Wider HV-Integer;\
        P1-;\
        P1O[26:24],Modify Colors,Off,GBA 2.2,GBA 1.6,NDS 1.6,VBA 1.4,75%,50%,25%;\
        P1-;\
        P1O[39],Sync core to video,On,Off;\
        P1O[10:9],Flickerblend,Off,Blend,30Hz;\
        P1O[22:21],2XResolution,Off,Background,Sprites,Both;\
        P1O[20],Spritelimit,Off,On;\
        P1-;\
        P1O[8:7],Stereo Mix,None,25%,50%,100%;\
        P1O[19],Fast Forward Sound,On,Off;\
        P2,Hardware;\
        P2-;\
        H6P2O[31:29],Solar Sensor,0%,15%,30%,42%,55%,70%,85%,100%;\
        H2P2O[16],Turbo,Off,On;\
        P2O[28],Homebrew BIOS(Reset!),Off,On;\
        P3,Miscellaneous;\
        P3-;\
        P3O[15:14],Storage,Auto,SDRAM,DDR3;\
        D5P3O[5],Pause when OSD is open,Off,On;\
        P3O[27],Rewind Capture,Off,On;\
        P3-;\
        P3-,Only Romhacks or Crash!;\
        P3O[40],GPIO HACK(RTC+Rumble),Off,On;\
        P3O[42:41],Underclock CPU,0,1,2,3;\
        - ;\
        R0,Reset;\
        J1,A,B,L,R,Select,Start,FastForward,Rewind,Savestates;\
        jn,A,B,L,R,Select,Start,X,X;\
        I,Load=DPAD Up|Save=Down|Slot=L+R,Active Slot 1,Active Slot 2,Active Slot 3,Active Slot 4,Save to state 1,Restore state 1,Save to state 2,Restore state 2,Save to state 3,Restore state 3,Save to state 4,Restore state 4,Rewinding...;\
        V,v230803"
    );
    assert!(config.is_ok(), "{:?}", config);
}
