//! TASBot File Formats readers and writers.
//!
//! This module provides readers and writers for TASBot file formats.

use std::fmt;
use std::io::Read;

/// NES controller input buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum R08InputButton {
    A = 0x80,
    B = 0x40,
    Select = 0x20,
    Start = 0x10,
    Up = 0x08,
    Down = 0x04,
    Left = 0x02,
    Right = 0x01,
    None = 0,
}

impl From<u8> for R08InputButton {
    fn from(value: u8) -> Self {
        match value {
            0x80 => Self::A,
            0x40 => Self::B,
            0x20 => Self::Select,
            0x10 => Self::Start,
            0x08 => Self::Up,
            0x04 => Self::Down,
            0x02 => Self::Left,
            0x01 => Self::Right,
            _ => Self::None,
        }
    }
}

impl R08InputButton {
    pub fn is_none(&self) -> bool {
        self == &Self::None
    }
}

/// The gamepad state of a frame.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NESGamepadState {
    buttons: u8,
}

impl fmt::Debug for NESGamepadState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NESGamepadState")
            .field(&format!("{:08b}", self.buttons))
            .field(
                &self
                    .buttons()
                    .into_iter()
                    .filter(|b| b != &R08InputButton::None)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl From<u8> for NESGamepadState {
    fn from(value: u8) -> Self {
        Self { buttons: value }
    }
}

impl NESGamepadState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn press(&mut self, button: R08InputButton) {
        self.buttons |= button as u8;
    }

    pub fn release(&mut self, button: R08InputButton) {
        self.buttons &= !(button as u8);
    }

    pub fn has(&self, button: R08InputButton) -> bool {
        self.buttons & (button as u8) != 0
    }

    pub fn buttons(&self) -> Vec<R08InputButton> {
        let mut buttons = Vec::new();
        for i in 0..8 {
            let mask = 1 << i;
            if self.buttons & mask != 0 {
                buttons.push(R08InputButton::from(mask));
            }
        }
        buttons
    }
}

/// A single frame of an R08 file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct R08Frame {
    pub player1: NESGamepadState,
    pub player2: NESGamepadState,
}

impl R08Frame {
    pub fn empty() -> Self {
        Self {
            player1: NESGamepadState::new(),
            player2: NESGamepadState::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct R08File {
    pub frames: Vec<R08Frame>,
}

impl R08File {
    pub fn empty() -> Self {
        Self { frames: Vec::new() }
    }

    pub fn read<R: Read>(mut reader: R) -> Result<Self, std::io::Error> {
        let mut frames = Vec::new();

        loop {
            let mut players = [0u8; 2];
            match reader.read(&mut players) {
                Ok(2) => {
                    frames.push(R08Frame {
                        player1: NESGamepadState {
                            buttons: players[0],
                        },
                        player2: NESGamepadState {
                            buttons: players[1],
                        },
                    });
                }
                Ok(0) => break,
                Ok(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid R08 file format",
                    ));
                }
                Err(e) => return Err(e),
            }
        }
        Ok(Self { frames })
    }

    pub fn iter(&self) -> std::slice::Iter<R08Frame> {
        self.frames.iter()
    }
}

#[test]
fn gamepad() {
    let mut gamepad = NESGamepadState::new();
    assert_eq!(gamepad.buttons(), [R08InputButton::None; 8]);

    gamepad.press(R08InputButton::A);
    assert_eq!(
        gamepad.buttons(),
        [
            R08InputButton::A,
            R08InputButton::None,
            R08InputButton::None,
            R08InputButton::None,
            R08InputButton::None,
            R08InputButton::None,
            R08InputButton::None,
            R08InputButton::None
        ]
    );

    let gamepad = NESGamepadState::from(0b0001_0001);
    assert_eq!(
        gamepad.buttons(),
        [
            R08InputButton::None,
            R08InputButton::None,
            R08InputButton::None,
            R08InputButton::Start,
            R08InputButton::None,
            R08InputButton::None,
            R08InputButton::None,
            R08InputButton::Right
        ]
    );
}
