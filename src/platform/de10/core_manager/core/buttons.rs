use bitvec::prelude::*;
use std::ops::Index;
use strum::{Display, EnumCount, EnumIter};

/// Buttons supported by the MisterFPGA API.
/// This is MiSTer specific.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Display, EnumCount, EnumIter)]
pub enum MisterFpgaButtons {
    // No mapping exists and the button should be discarded or ignored.
    NoMapping = -1,

    DpadRight = 0,
    DpadLeft,
    DpadDown,
    DpadUp,
    B,
    A,
    Y,
    X,
    LeftShoulder,
    RightShoulder,
    Back,
    Start,
    _Null12 = 12,
    _Null13,
    _Null14,
    _Null15,
    _Null16,
    _Null17,
    _Null18,
    _Null19,
    _Null20,
    Guide,
    Guide2,
    Menu,
    LeftX,
    LeftY,
    RightX,
    RightY,
    AsysX,
    AsysY,
    Null30,
    Null31,
}

pub struct ButtonMapping {
    map: Vec<MisterFpgaButtons>,
}

impl Index<usize> for ButtonMapping {
    type Output = MisterFpgaButtons;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.map[index]
    }
}

impl<const N: usize> From<[MisterFpgaButtons; N]> for ButtonMapping {
    fn from(value: [MisterFpgaButtons; N]) -> Self {
        Self {
            map: value.to_vec(),
        }
    }
}

impl ButtonMapping {
    pub fn new(capacity: usize) -> Self {
        Self {
            map: vec![MisterFpgaButtons::NoMapping; capacity],
        }
    }

    /// Create a default SDL mapping to MisterFPGA buttons, naively.
    pub fn sdl() -> Self {
        [
            MisterFpgaButtons::B,
            MisterFpgaButtons::A,
            MisterFpgaButtons::Y,
            MisterFpgaButtons::X,
            MisterFpgaButtons::Back,
            MisterFpgaButtons::Guide,
            MisterFpgaButtons::Start,
            MisterFpgaButtons::LeftX,
            MisterFpgaButtons::RightX,
            MisterFpgaButtons::LeftShoulder,
            MisterFpgaButtons::RightShoulder,
            MisterFpgaButtons::DpadUp,
            MisterFpgaButtons::DpadDown,
            MisterFpgaButtons::DpadLeft,
            MisterFpgaButtons::DpadRight,
            MisterFpgaButtons::NoMapping,
            MisterFpgaButtons::NoMapping,
            MisterFpgaButtons::NoMapping,
            MisterFpgaButtons::NoMapping,
            MisterFpgaButtons::NoMapping,
            MisterFpgaButtons::NoMapping,
        ]
        .into()
    }

    pub fn map(&mut self, from: usize, to: MisterFpgaButtons) {
        self.map[from] = to;
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct ButtonMap(BitArray<[u32; 1], Lsb0>);

impl ButtonMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn down(&mut self, btn: MisterFpgaButtons) {
        if btn == MisterFpgaButtons::NoMapping {
            return;
        }
        self.0.set(btn as usize, true);
    }
    pub fn up(&mut self, btn: MisterFpgaButtons) {
        if btn == MisterFpgaButtons::NoMapping {
            return;
        }
        self.0.set(btn as usize, false);
    }

    pub fn set(&mut self, v: u32) {
        self.0.store(v);
    }
    pub fn mask(&self) -> u32 {
        self.0.load()
    }
}
