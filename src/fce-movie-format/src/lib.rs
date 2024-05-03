use std::collections::BTreeMap;
use std::fmt::Debug;
use std::io::BufRead;
use std::str::FromStr;

use base64::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FceError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid FCE version header: {0}")]
    VersionParseError(std::num::ParseIntError),

    #[error("Invalid rerecord count header: {0}")]
    RerecordCountParseError(std::num::ParseIntError),

    #[error("Invalid PAL header: {0}")]
    PalParseError(std::num::ParseIntError),

    #[error("Invalid new PPU header: {0}")]
    NewPpuParseError(std::num::ParseIntError),

    #[error("Invalid FDS header: {0}")]
    FdsParseError(std::num::ParseIntError),

    #[error("Invalid Four Score header: {0}")]
    FourScoreParseError(std::num::ParseIntError),

    #[error("Invalid microphone header: {0}")]
    MicrophoneParseError(std::num::ParseIntError),

    #[error("Invalid binary header: {0}")]
    BinaryParseError(std::num::ParseIntError),

    #[error("Invalid length header: {0}")]
    LengthParseError(std::num::ParseIntError),

    #[error("Invalid savestate header: {0}")]
    SavestateParseError(hex::FromHexError),

    #[error("Invalid port header value. Must be SI_NONE, SI_GAMEPAD or SI_ZAPPER (or 0, 1 or 2).")]
    InvalidPortHeader(),

    #[error("Invalid GUID: {0}")]
    GuidParseError(hex::FromHexError),

    #[error("Invalid version: {0}")]
    InvalidVersion(u8),

    #[error("Invalid checksum header: {0}")]
    InvalidRomChecksum(base64::DecodeError),

    #[error("Missing subtitle frame number")]
    MissingSubtitleFrameNumber,

    #[error("Invalid subtitle frame number: {0}")]
    InvalidFrameNumber(std::num::ParseIntError),

    #[error("Invalid input line: {0}")]
    InvalidInputLine(String),

    #[error("Invalid input line for Gamepad: {0}")]
    InvalidGamepadInputLine(String),
}

#[derive(Debug, Clone)]
pub struct FceHeader {
    pub version: u8,
    pub emu_version: String,
    pub rerecord_count: Option<u32>,
    pub pal: bool,
    pub new_ppu: bool,
    pub fds: bool,
    pub fourscore: bool,
    pub microphone: bool,
    pub binary: bool,
    pub length: Option<u32>,
    pub port0: FceInputPortType,
    pub port1: FceInputPortType,
    pub port2: FceInputPortType,
    pub rom_filename: String,
    pub rom_checksum: [u8; 16],
    pub guid: [u8; 16],
    pub savestate: Option<Vec<u8>>,

    pub comments: BTreeMap<String, Vec<String>>,
    pub subtitles: BTreeMap<u32, String>,
}

impl FceHeader {
    pub(crate) fn new() -> Self {
        Self {
            version: 0,
            emu_version: String::new(),
            rerecord_count: None,
            pal: false,
            new_ppu: false,
            fds: false,
            fourscore: false,
            microphone: false,
            binary: false,
            length: None,
            port0: FceInputPortType::None,
            port1: FceInputPortType::None,
            port2: FceInputPortType::None,
            rom_filename: String::new(),
            rom_checksum: [0; 16],
            guid: [0; 16],
            savestate: None,
            comments: Default::default(),
            subtitles: Default::default(),
        }
    }
}

/// NES controller input buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FceInputButton {
    A = 0x01,
    B = 0x02,
    Select = 0x04,
    Start = 0x08,
    Up = 0x10,
    Down = 0x20,
    Left = 0x40,
    Right = 0x80,
}

impl From<u8> for FceInputButton {
    fn from(value: u8) -> Self {
        match value {
            0x01 => FceInputButton::A,
            0x02 => FceInputButton::B,
            0x04 => FceInputButton::Select,
            0x08 => FceInputButton::Start,
            0x10 => FceInputButton::Up,
            0x20 => FceInputButton::Down,
            0x40 => FceInputButton::Left,
            0x80 => FceInputButton::Right,
            _ => panic!("Invalid FceInputButton value: {}", value),
        }
    }
}

/// NES controller input gamepad state.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct FceInputGamepad(u8);

impl Debug for FceInputGamepad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set()
            .entries(
                (0u8..8)
                    .filter(|i| self.0 & (1 << i) != 0)
                    .map(|i| FceInputButton::from(1 << i)),
            )
            .finish()
    }
}

impl FromStr for FceInputGamepad {
    type Err = FceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 8 {
            return Err(FceError::InvalidGamepadInputLine(s.to_string()));
        }

        // RLDUTSBA
        let gamepad = s
            .chars()
            .map(|c| c != ' ' && c != '.')
            .rev()
            .enumerate()
            .filter(|(_, c)| *c)
            .fold(0, |acc, (i, _)| acc | (1u8 << i));

        Ok(Self(gamepad))
    }
}

impl FceInputGamepad {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn set(&mut self, button: FceInputButton) {
        self.0 |= button as u8;
    }

    pub fn clear(&mut self, button: FceInputButton) {
        self.0 &= !(button as u8);
    }

    pub fn has(&self, button: FceInputButton) -> bool {
        self.0 & (button as u8) != 0
    }

    pub fn buttons(&self) -> Vec<FceInputButton> {
        (0..8)
            .filter(|i| self.0 & (1 << *i) != 0)
            .map(|i| FceInputButton::from(1 << i))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FceInputZapper {
    x: u16,
    y: u16,
    mouse: bool,
    internal: u8,
    z: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FceInputPort {
    None,
    Gamepad(FceInputGamepad),
    Zapper(FceInputZapper),
}

impl FceInputPort {
    pub fn empty(ty: FceInputPortType) -> Self {
        match ty {
            FceInputPortType::None => FceInputPort::None,
            FceInputPortType::Gamepad => FceInputPort::Gamepad(FceInputGamepad::new()),
            FceInputPortType::Zapper => FceInputPort::Zapper(FceInputZapper {
                x: 0,
                y: 0,
                mouse: false,
                internal: 0,
                z: 0,
            }),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, FceInputPort::None)
    }

    pub fn as_gamepad(&self) -> Option<&FceInputGamepad> {
        match self {
            FceInputPort::Gamepad(gamepad) => Some(gamepad),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FceInputPortType {
    None,
    Gamepad,
    Zapper,
}

impl FromStr for FceInputPortType {
    type Err = FceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" | "SI_NONE" => Ok(FceInputPortType::None),
            "1" | "SI_GAMEPAD" => Ok(FceInputPortType::Gamepad),
            "2" | "SI_ZAPPER" => Ok(FceInputPortType::Zapper),
            _ => Err(FceError::InvalidPortHeader()),
        }
    }
}

impl From<FceInputPort> for FceInputPortType {
    fn from(port: FceInputPort) -> Self {
        match port {
            FceInputPort::None => FceInputPortType::None,
            FceInputPort::Gamepad(_) => FceInputPortType::Gamepad,
            FceInputPort::Zapper(_) => FceInputPortType::Zapper,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FceFrameCommand {
    SoftReset = 0x01,
    HardReset = 0x02,
    FdsDiskInsert = 0x04,
    FdsDiskEject = 0x08,
    VsInsertCoin = 0x10,
}

impl From<u8> for FceFrameCommand {
    fn from(value: u8) -> Self {
        match value {
            0x01 => FceFrameCommand::SoftReset,
            0x02 => FceFrameCommand::HardReset,
            0x04 => FceFrameCommand::FdsDiskInsert,
            0x08 => FceFrameCommand::FdsDiskEject,
            0x10 => FceFrameCommand::VsInsertCoin,
            _ => panic!("Invalid FceFrameCommand value: {}", value),
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct FceFrameCommandSet(u8);

impl Debug for FceFrameCommandSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set()
            .entries(
                (0u8..8)
                    .filter(|i| self.0 & (1 << i) != 0)
                    .map(|i| FceFrameCommand::from(1 << i)),
            )
            .finish()
    }
}

impl FceFrameCommandSet {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn set(&mut self, command: FceFrameCommand) {
        self.0 |= command as u8;
    }

    pub fn clear(&mut self, command: FceFrameCommand) {
        self.0 &= !(command as u8);
    }

    pub fn has(&self, command: FceFrameCommand) -> bool {
        self.0 & (command as u8) != 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FceFrame {
    pub commands: FceFrameCommandSet,
    pub port0: Option<FceInputPort>,
    pub port1: Option<FceInputPort>,
    pub port2: Option<FceInputPort>,
}

impl FceFrame {
    pub fn empty(fce_header: &FceHeader) -> Self {
        let port0 = FceInputPort::empty(fce_header.port0);
        let port1 = FceInputPort::empty(fce_header.port1);
        let port2 = FceInputPort::empty(fce_header.port2);

        Self {
            commands: FceFrameCommandSet::new(),
            port0: if port0.is_none() { None } else { Some(port0) },
            port1: if port1.is_none() { None } else { Some(port1) },
            port2: if port2.is_none() { None } else { Some(port2) },
        }
    }
}

fn parse_header_(header: &mut FceHeader, line: String) -> Result<(), FceError> {
    if line == "|" {
        return Ok(());
    }

    if let Some((key, value)) = line.trim().split_once(' ') {
        match key {
            "version" => {
                let v = value.parse().map_err(FceError::VersionParseError)?;
                if !(2..=3).contains(&v) {
                    return Err(FceError::InvalidVersion(v));
                }

                header.version = v;
            }
            "emuVersion" | "emu_version" => header.emu_version = value.to_string(),
            "rerecordCount" | "rerecord_count" => {
                header.rerecord_count = Some(
                    value
                        .parse::<u32>()
                        .map_err(FceError::RerecordCountParseError)?,
                )
            }
            "palFlag" | "pal" => {
                header.pal = value.parse::<u8>().map_err(FceError::PalParseError)? != 0
            }
            "NewPPU" | "newPPU" | "new_ppu" => {
                header.new_ppu = value.parse::<u8>().map_err(FceError::NewPpuParseError)? != 0
            }
            "FDS" | "fds" => {
                header.fds = value.parse::<u8>().map_err(FceError::FdsParseError)? != 0
            }
            "fourscore" => {
                header.fourscore = value.parse::<u8>().map_err(FceError::FourScoreParseError)? != 0
            }
            "microphone" => {
                header.microphone = value
                    .parse::<u8>()
                    .map_err(FceError::MicrophoneParseError)?
                    != 0
            }
            "binary" => {
                header.binary = value.parse::<u8>().map_err(FceError::BinaryParseError)? != 0
            }
            "length" => header.length = Some(value.parse().map_err(FceError::LengthParseError)?),
            "comment" => {
                let (subject, comment) = value.split_once(' ').unwrap_or(("", value));
                header
                    .comments
                    .entry(subject.to_string())
                    .or_default()
                    .push(comment.to_string());
            }
            "port0" => header.port0 = value.parse()?,
            "port1" => header.port1 = value.parse()?,
            "port2" => header.port2 = value.parse()?,
            "romFilename" | "rom_filename" => header.rom_filename = value.to_string(),
            "romChecksum" | "rom_checksum" => {
                if let Some(b64) = value.strip_prefix("base64:") {
                    let checksum = BASE64_STANDARD
                        .decode(b64)
                        .map_err(FceError::InvalidRomChecksum)?;
                    let checksum: [u8; 16] = checksum.as_slice().try_into().map_err(|_e| {
                        FceError::InvalidRomChecksum(base64::DecodeError::InvalidLength(
                            value.len(),
                        ))
                    })?;
                    header.rom_checksum = checksum;
                } else {
                    return Err(FceError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid romChecksum format",
                    )));
                }
            }
            "guid" => {
                let mut guid = [0; 16];
                hex::decode_to_slice(value.replace('-', ""), &mut guid)
                    .map_err(FceError::GuidParseError)?;
                header.guid = guid;
            }
            "savestate" => {
                if let Some(hexa) = value.strip_prefix("0x") {
                    header.savestate =
                        Some(hex::decode(hexa).map_err(FceError::SavestateParseError)?);
                } else {
                    return Err(FceError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid savestate format",
                    )));
                }
            }
            "subtitle" => {
                let (frame, subtitle) = value
                    .split_once(' ')
                    .ok_or_else(|| FceError::MissingSubtitleFrameNumber)?;
                header.subtitles.insert(
                    frame.parse().map_err(FceError::InvalidFrameNumber)?,
                    subtitle.to_string(),
                );
            }
            unknown => {
                tracing::warn!("Unknown FCE header key: {}", unknown);
            }
        }

        Ok(())
    } else {
        Err(FceError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid FCE header line",
        )))
    }
}

fn parse_port(ty: FceInputPortType, inner: &str) -> Result<Option<FceInputPort>, FceError> {
    if inner.is_empty() {
        return Ok(None);
    }

    match ty {
        FceInputPortType::None => Ok(None),
        FceInputPortType::Gamepad => Ok(Some(FceInputPort::Gamepad(inner.parse()?))),
        FceInputPortType::Zapper => todo!("Zapper input not supported yet"),
    }
}

fn parse_input_line(header: &FceHeader, line: &str) -> Result<FceFrame, FceError> {
    let mut parts = line.split('|');
    if line.starts_with('|') {
        parts.next();
    }

    let commands = parts.next().unwrap();
    let port0 = parse_port(header.port0, parts.next().unwrap())?;
    let port1 = parse_port(header.port1, parts.next().unwrap())?;
    let port2 = parse_port(header.port2, parts.next().unwrap())?;

    Ok(FceFrame {
        commands: FceFrameCommandSet(commands.parse::<u8>().unwrap()),
        port0,
        port1,
        port2,
    })
}

pub struct FceFrameInputs(Vec<FceFrame>);

impl FceFrameInputs {
    pub fn iter(&self) -> impl Iterator<Item = &FceFrame> {
        self.0.iter()
    }
}

pub struct FceFile {
    pub header: FceHeader,
    pub inputs: FceFrameInputs,
}

impl FceFile {
    pub fn load_stream(mut input: impl BufRead) -> Result<Self, FceError> {
        let mut header = FceHeader::new();

        let mut headers = Vec::with_capacity(1024);
        input.read_until(b'|', &mut headers)?;
        for line_buff in headers.split(|c| c == &b'\n') {
            let line = String::from_utf8_lossy(line_buff).to_string();
            parse_header_(&mut header, line)?;
        }

        let mut inputs = Vec::with_capacity(1024);

        if header.binary {
            todo!("Binary FCE format not supported yet");
        } else {
            for line in input.lines() {
                let line = line?;
                inputs.push(parse_input_line(&header, &line)?);
            }
        }

        Ok(Self {
            header,
            inputs: FceFrameInputs(inputs),
        })
    }

    pub fn frames(&self) -> impl Iterator<Item = &FceFrame> {
        self.inputs.iter()
    }
}
