use bitvec::prelude::*;
use std::fmt::{Debug, Formatter};
use std::ops::Index;
use std::str::FromStr;
use strum::{Display, EnumCount, EnumIter, EnumString};

/// Buttons supported by the MisterFPGA API.
/// This is MiSTer specific.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Display, EnumCount, EnumIter, EnumString)]
pub enum MisterFpgaButtons {
    // No mapping exists and the button should be discarded or ignored.
    NoMapping = -1,

    DpadRight = 0,
    DpadLeft,
    DpadDown,
    DpadUp,
    A,
    B,
    X,
    Y,
    LeftShoulder,
    RightShoulder,
    Back,
    Start,
    MsRight = 12,
    MsLeft,
    MsDown,
    MsUp,
    MsBtnL,
    MsBtnR,
    MsBtnM,
    MsBtnEmu,
    BtnOsdKtglKb = 20,
    BtnOsdKtglGamepad1,
    BtnOsdKtglGamepad2,
    Menu,
    Axis1X,
    Axis1Y,
    Axis2X,
    Axis2Y,
    AxisX,
    AxisY,
    AxisMX,
    AxisMY,
}

pub struct ButtonMapping {
    map: Vec<MisterFpgaButtons>,
}

impl Default for ButtonMapping {
    fn default() -> Self {
        Self::sdl()
    }
}

impl Debug for ButtonMapping {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.map.iter().enumerate()).finish()
    }
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
            MisterFpgaButtons::BtnOsdKtglGamepad1,
            MisterFpgaButtons::Start,
            MisterFpgaButtons::Axis1X,
            MisterFpgaButtons::Axis2X,
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

    pub fn from_snes_list(mut mapping: &[impl AsRef<str>]) -> Self {
        let mut inner = [MisterFpgaButtons::NoMapping; 16];
        let mut i = 0;
        while i < 11 {
            if let Some(btn) = mapping.get(i) {
                inner[i] = MisterFpgaButtons::from_str(btn.as_ref())
                    .unwrap_or(MisterFpgaButtons::NoMapping);
                i += 1;
            } else {
                break;
            }
        }
        inner[11..15].copy_from_slice(&[
            MisterFpgaButtons::DpadUp,
            MisterFpgaButtons::DpadDown,
            MisterFpgaButtons::DpadLeft,
            MisterFpgaButtons::DpadRight,
        ]);
        inner.into()
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

#[test]
fn test_button_nes() {
    let mapping = ButtonMapping::from_snes_list(&vec!["A", "B", "Select", "Start"]);
    assert_eq!(mapping[0], MisterFpgaButtons::A);
    assert_eq!(mapping[1], MisterFpgaButtons::B);
}
