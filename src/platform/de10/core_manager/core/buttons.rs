use bitvec::prelude::*;
use std::ops::Index;
use strum::{Display, EnumCount, EnumIter};

/// Buttons supported by the MisterFPGA API.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Display, EnumCount, EnumIter)]
pub enum MisterButtons {
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
    map: Vec<MisterButtons>,
}

impl Index<usize> for ButtonMapping {
    type Output = MisterButtons;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.map[index]
    }
}

impl<const N: usize> From<[MisterButtons; N]> for ButtonMapping {
    fn from(value: [MisterButtons; N]) -> Self {
        Self {
            map: value.to_vec(),
        }
    }
}

impl ButtonMapping {
    pub fn new(capacity: usize) -> Self {
        Self {
            map: vec![MisterButtons::NoMapping; capacity],
        }
    }

    /// Create a default SDL mapping to MisterFPGA buttons, naively.
    pub fn sdl() -> Self {
        [
            MisterButtons::B,
            MisterButtons::A,
            MisterButtons::Y,
            MisterButtons::X,
            MisterButtons::Back,
            MisterButtons::Guide,
            MisterButtons::Start,
            MisterButtons::LeftX,
            MisterButtons::RightX,
            MisterButtons::LeftShoulder,
            MisterButtons::RightShoulder,
            MisterButtons::DpadUp,
            MisterButtons::DpadDown,
            MisterButtons::DpadLeft,
            MisterButtons::DpadRight,
            MisterButtons::NoMapping,
            MisterButtons::NoMapping,
            MisterButtons::NoMapping,
            MisterButtons::NoMapping,
            MisterButtons::NoMapping,
            MisterButtons::NoMapping,
        ]
        .into()
    }

    pub fn map(&mut self, from: usize, to: MisterButtons) {
        self.map[from] = to;
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct ButtonMap(BitArray<[u32; 1], Lsb0>);

impl ButtonMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn down(&mut self, btn: MisterButtons) {
        if btn == MisterButtons::NoMapping {
            return;
        }
        self.0.set(btn as usize, true);
    }
    pub fn up(&mut self, btn: MisterButtons) {
        if btn == MisterButtons::NoMapping {
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
