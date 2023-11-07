//! Parses the config string of a core, and extract the proper parameters.
//! See this documentation for more information:
//! https://github.com/MiSTer-devel/MkDocs_MiSTer/blob/main/docs/developer/conf_str.md
//!
//! This is located in utils to allow to run test. There is no FPGA or MiSTer specific
//! code in this module, even though it isn't used outside of MiSTer itself.
use crate::types::StatusBitMap;
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use std::ops::Range;
use std::path::Path;
use std::str::FromStr;
use tracing::{debug, info};

pub mod midi;
pub mod settings;
pub mod uart;

static LABELED_SPEED_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d*)(\([^)]*\))?").unwrap());
static LOAD_FILE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(S)?(\d)?,((?:\w{3})+)(?:,([^,]*))?(?:,([0-9a-fA-F]*))?").unwrap());
static MOUNT_SD_CARD_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\d),((?:\w{3})+)(?:,([^,]*))?").unwrap());
static OPTION_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([0-9A-V])([0-9A-V])?,([^,]*),(.*)").unwrap());

fn parse_bit_index_(s: &str, high: bool) -> Option<u8> {
    let i = u8::from_str_radix(s, 32).ok()?;
    Some(if high { i + 32 } else { i })
}

#[derive(Debug, Clone)]
pub struct LoadFileInfo {
    /// Core supports save files, load a file, and mount a save for reading or writing
    save_support: bool,

    /// Explicit index (or index is generated from line number if index not given).
    index: u8,

    /// A concatenated list of 3 character file extensions. For example, BINGEN would be
    /// BIN and GEN extensions.
    extensions: Vec<String>,

    /// Optional {Text} string is the text that is displayed before the extensions like
    /// "Load RAM". If {Text} is not specified, then default is "Load *".
    label: Option<String>,

    /// Optional {Address} - load file directly into DDRAM at this address.
    /// ioctl_index from hps_io will be: ioctl_index[5:0] = index(explicit or auto),
    /// ioctl_index[7:6] = extension index
    address: Option<usize>,
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
    LoadFileAndRemember {
        save_support: bool,
        index: u8,
        extensions: Vec<String>,
        label: Option<String>,
        address: Option<usize>,
    },

    /// Mount SD card menu option.
    MountSdCard {
        slot: u8,
        extensions: Vec<String>,
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

    SnesButtonList {
        buttons: Vec<String>,
    },

    SnesButtonDefaultList {
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

        /// The label of the page.
        label: String,

        /// The menu items on the page.
        items: Vec<ConfigMenu>,
    },

    /// `I,INFO1,INFO2,...,INFO255` - INFO1-INFO255 lines to display as OSD info (top left
    /// corner of screen).
    Info(Vec<String>),

    /// `V,{Version String}` - Version string. {Version String} is the version string.
    /// Takes the core name and appends version string for name to display.
    Version(String),
}

impl ConfigMenu {
    fn load_file_remember(s: &str, default_index: u8) -> Result<Self, &'static str> {
        if let Self::LoadFile(inner) = Self::load_file(s, default_index)? {
            let LoadFileInfo {
                save_support,
                index,
                extensions,
                label,
                address,
            } = *inner;
            Ok(Self::LoadFileAndRemember {
                save_support,
                index,
                extensions,
                label,
                address,
            })
        } else {
            Err("Invalid load file (remember) string.")
        }
    }

    fn load_file(s: &str, default_index: u8) -> Result<Self, &'static str> {
        let captures = LOAD_FILE_RE
            .captures(s)
            .ok_or("Invalid load file string.")?;

        let save_support = captures.get(1).is_some();
        let index = captures
            .get(2)
            .map(|s| s.as_str().parse::<u8>().unwrap_or(0))
            .unwrap_or(default_index);
        let extensions = captures
            .get(3)
            .ok_or("Invalid extension list")?
            .as_str()
            .chars()
            .chunks(3)
            .into_iter()
            .map(|chunk| chunk.collect::<String>())
            .collect::<Vec<_>>();
        let label = captures.get(4).map(|s| s.as_str().to_string());
        let address = captures
            .get(5)
            .map(|s| usize::from_str_radix(s.as_str(), 16).unwrap_or(0));

        Ok(Self::LoadFile(Box::new(LoadFileInfo {
            save_support,
            index,
            extensions,
            label,
            address,
        })))
    }

    fn sd_card(s: &str) -> Result<Self, &'static str> {
        let captures = MOUNT_SD_CARD_RE
            .captures(s)
            .ok_or("Invalid SD card string.")?;

        let slot = captures
            .get(1)
            .ok_or("Invalid SD card slot")?
            .as_str()
            .parse::<u8>()
            .map_err(|_| "Invalid SD card slot")?;
        let extensions = captures
            .get(2)
            .ok_or("Invalid extension list")?
            .as_str()
            .chars()
            .chunks(3)
            .into_iter()
            .map(|chunk| chunk.collect::<String>())
            .collect::<Vec<_>>();
        let label = captures.get(3).map(|s| s.as_str().to_string());

        Ok(Self::MountSdCard {
            slot,
            extensions,
            label,
        })
    }

    fn option(s: &str, high: bool) -> Result<Self, &'static str> {
        let captures = OPTION_RE.captures(s).ok_or("Invalid option string.")?;

        let index1 = match captures.get(1) {
            Some(s) => parse_bit_index_(s.as_str(), high).ok_or("Invalid index.")?,
            None => Err("Invalid index start.")?,
        };
        let index2 = match captures.get(2) {
            Some(s) => parse_bit_index_(s.as_str(), high).ok_or("Invalid index.")?,
            None => index1,
        };

        let label = captures
            .get(3)
            .ok_or("No label specified")?
            .as_str()
            .to_string();

        let choices = captures
            .get(4)
            .ok_or("No choices specified")?
            .as_str()
            .split(',')
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        Ok(Self::Option {
            bits: index1..(index2 + 1),
            label,
            choices,
        })
    }

    fn trigger(s: &str, high: bool, close: bool) -> Result<Self, &'static str> {
        let index = parse_bit_index_(&s[..1], high).ok_or("Invalid index.")?;
        let label = s[2..].to_string();

        Ok(Self::Trigger {
            close_osd: close,
            index,
            label,
        })
    }
}

impl ConfigMenu {
    pub fn parse_line(line: &str, index: u8) -> Result<Self, &'static str> {
        #[inline]
        fn to_str(s: &[u8]) -> &str {
            unsafe { std::str::from_utf8_unchecked(s) }
        }

        #[inline]
        fn to_string(s: &[u8]) -> String {
            to_str(s).to_string()
        }

        #[inline]
        fn to_u32(s: &[u8]) -> Option<u32> {
            to_str(s).parse::<u32>().ok()
        }

        match line.as_bytes() {
            [b'-'] => Ok(ConfigMenu::Empty(None)),
            [b'-', rest @ ..] => Ok(ConfigMenu::Empty(Some(to_string(rest)))),
            [b'C', b',', label @ ..] => Ok(ConfigMenu::Cheat(Some(to_string(label)))),
            [b'C'] => Ok(ConfigMenu::Cheat(None)),

            [b'D', b'I', b'P'] => Ok(ConfigMenu::Dip),

            [b'D', d, rest @ ..] => to_u32(&[*d])
                .ok_or("Invalid D index")
                .map(|d| Self::DisableIf(d, Self::parse_line(to_str(rest), index).unwrap().into())),
            [b'd', d, rest @ ..] => to_u32(&[*d]).ok_or("Invalid d index").map(|d| {
                Self::DisableUnless(d, Self::parse_line(to_str(rest), index).unwrap().into())
            }),
            [b'H', h, rest @ ..] => to_u32(&[*h])
                .ok_or("Invalid H index")
                .map(|h| Self::HideIf(h, Self::parse_line(to_str(rest), index).unwrap().into())),
            [b'h', h, rest @ ..] => to_u32(&[*h]).ok_or("Invalid h index").map(|h| {
                Self::HideUnless(h, Self::parse_line(to_str(rest), index).unwrap().into())
            }),

            [b'F', b'C', rest @ ..] => Self::load_file_remember(to_str(rest), index),
            [b'F', rest @ ..] => Self::load_file(to_str(rest), index),
            [b'S', rest @ ..] => Self::sd_card(to_str(rest)),

            [b'O', rest @ ..] => Self::option(to_str(rest), false),
            [b'o', rest @ ..] => Self::option(to_str(rest), true),
            [b'R', rest @ ..] => Self::trigger(to_str(rest), false, true),
            [b'r', rest @ ..] => Self::trigger(to_str(rest), true, true),
            [b'T', rest @ ..] => Self::trigger(to_str(rest), false, false),
            [b't', rest @ ..] => Self::trigger(to_str(rest), true, false),

            [b'I', rest @ ..] => Ok(Self::Info(
                to_str(rest)
                    .split(',')
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            )),

            [b'P', number, rest @ ..] => {
                let index = to_u32(&[*number]).ok_or("Invalid page index")?;
                let label = to_string(rest);
                Ok(ConfigMenu::Page {
                    index: index as u8,
                    label,
                    items: Vec::new(),
                })
            }

            [b'J', b'1', b',', buttons @ ..] => Ok(ConfigMenu::JoystickButtons {
                keyboard: true,
                buttons: to_str(buttons)
                    .split(',')
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            }),
            [b'J', b',', buttons @ ..] => Ok(ConfigMenu::JoystickButtons {
                keyboard: false,
                buttons: to_str(buttons)
                    .split(',')
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            }),
            [b'j', b'n', rest @ ..] => Ok(ConfigMenu::SnesButtonList {
                buttons: to_str(rest)
                    .split(',')
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            }),
            [b'j', b'p', rest @ ..] => Ok(ConfigMenu::SnesButtonDefaultList {
                buttons: to_str(rest)
                    .split(',')
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            }),

            [b'V', b',', rest @ ..] => Ok(Self::Version(to_string(rest))),

            _ => {
                debug!(?line, "Unknown menu option");
                eprintln!("line {line:?}");
                Err("Unknown menu option")
            }
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
    pub fn status_bit_map_mask(&self) -> StatusBitMap {
        let mut arr = StatusBitMap::new();
        // First bit is always 1 for soft reset (reserved).
        arr.set(0, true);

        for item in self.menu.iter() {
            if let ConfigMenu::Option { ref bits, .. } = item {
                for i in bits.clone() {
                    arr.set(i as usize, true);
                }
            }
        }

        arr
    }

    pub fn load_info(&self, path: impl AsRef<Path>) -> Result<Option<LoadFileInfo>, String> {
        let ext = match path.as_ref().extension() {
            Some(ext) => ext.to_string_lossy(),
            None => return Err("No extension".to_string()),
        };

        for item in self.menu.iter() {
            if let ConfigMenu::LoadFile(ref info) = item {
                if info.extensions.iter().any(|ext| ext == ext.as_str()) {
                    return Ok(Some(info.as_ref().clone()));
                }
            }
        }
        Ok(None)
    }

    pub fn version(&self) -> Option<&str> {
        for item in self.menu.iter() {
            if let ConfigMenu::Version(ref version) = item {
                return Some(version);
            }
        }
        None
    }
}

impl FromStr for Config {
    type Err = &'static str;

    fn from_str(cfg_string: &str) -> Result<Self, Self::Err> {
        if cfg_string.is_empty() {
            return Err("Empty config string.");
        }

        let name: String;
        let mut menu = Vec::new();
        let mut settings = settings::Settings::default();

        info!("Parsing config string: {}", cfg_string);

        let mut iter = cfg_string.split(';');
        // First element is always the name.
        if let Some(core_name) = iter.next() {
            name = core_name.to_string();
        } else {
            return Err("Empty config string");
        }

        // Next element is always the various settings.
        if let Some(core_settings) = iter.next() {
            if !core_settings.is_empty() {
                settings = settings::Settings::from_str(core_settings)?;
            }
        } else {
            return Err("No settings in config string");
        }

        // Rest is menu options.
        for (i, item) in iter.enumerate() {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            menu.push(ConfigMenu::parse_line(item, i as u8)?);
        }

        Ok(Self {
            name,
            settings,
            menu,
        })
    }
}

#[test]
fn config_string_nes() {
    // Taken from https://github.com/MiSTer-devel/NES_MiSTer/blob/7a645f4/NES.sv#L219
    let config = Config::from_str("\
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
        V,v123456"
    );

    assert!(config.is_ok(), "{:?}", config);
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
