use std::fmt::Debug;
use std::str::FromStr;

use bitvec::prelude::*;
use fixed_map::Key;
use static_assertions::{const_assert, const_assert_eq};
use strum::{Display, EnumCount, EnumIter, EnumString, FromRepr, IntoEnumIterator};
use tracing::trace;

/// Buttons supported by the MisterFPGA API.
/// This is MiSTer specific.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Display, EnumCount, EnumIter, EnumString, Key, FromRepr,
)]
#[repr(i8)]
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

    #[strum(serialize = "L")]
    LeftShoulder,
    #[strum(serialize = "R")]
    RightShoulder,

    #[strum(serialize = "Select", serialize = "Back")]
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

/// A pressed button map. This receives SDL button index and create a bits
/// array of pressed buttons that can be sent to MiSTer.
#[derive(Debug, Copy, Clone)]
pub struct ButtonMap {
    /// The map of SDL Button Index (`u8`) to MisterFpgaButtons.
    /// Since there's only 256 buttons, we can use 256 bytes to store
    /// the mapping.
    map: [MisterFpgaButtons; 256],

    /// The map of MisterFpgaButtons to indices in the core (`u8`).
    core_map: fixed_map::Map<MisterFpgaButtons, u8>,

    /// The bits array of pressed buttons. In a SNES controller, there
    /// are 16 buttons.
    bits: BitArray<[u32; 1], Lsb0>,
}

const_assert!(std::mem::size_of::<MisterFpgaButtons>() == 1);
const_assert_eq!(std::mem::size_of::<[Option<u8>; 256]>(), 512);

impl<const N: usize> From<[MisterFpgaButtons; N]> for ButtonMap {
    fn from(value: [MisterFpgaButtons; N]) -> Self {
        let mut this = Self::new();
        for (i, btn) in value.iter().enumerate() {
            this.add_mapping(i as u8, *btn);
        }
        this
    }
}

impl Default for ButtonMap {
    fn default() -> Self {
        Self::map_from_snes_list(&["A", "B", "X", "Y", "L", "R", "Back", "Start"])
    }
}

impl ButtonMap {
    pub fn new() -> Self {
        let mut this = Self {
            map: [MisterFpgaButtons::NoMapping; 256],
            // The default SNES -> Core map is simply i => i.
            core_map: MisterFpgaButtons::iter()
                .filter(|x| x != &MisterFpgaButtons::NoMapping)
                .enumerate()
                .map(|(i, btn)| (btn, i as u8))
                .collect::<fixed_map::Map<_, _>>(),
            bits: BitArray::ZERO,
        };

        // This is the default SDL -> SNES map.
        this.map[0..15].clone_from_slice(&[
            MisterFpgaButtons::A,
            MisterFpgaButtons::B,
            MisterFpgaButtons::X,
            MisterFpgaButtons::Y,
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
        ]);

        this
    }

    pub fn map_from_snes_list(list: &[&str]) -> Self {
        let mut map = Self::new();
        // Clear the SNES -> Index map, we only will support what's passed in.
        map.core_map = fixed_map::Map::new();
        map.add_mapping(0, MisterFpgaButtons::DpadRight);
        map.add_mapping(1, MisterFpgaButtons::DpadLeft);
        map.add_mapping(2, MisterFpgaButtons::DpadDown);
        map.add_mapping(3, MisterFpgaButtons::DpadUp);

        for (i, name) in list.iter().enumerate() {
            if let Ok(btn) = MisterFpgaButtons::from_str(name.trim()) {
                map.add_mapping((i + 4) as u8, btn);
            }
        }

        map
    }

    pub fn add_mapping(&mut self, index: u8, mister_btn: MisterFpgaButtons) {
        if mister_btn != MisterFpgaButtons::NoMapping {
            trace!(?index, ?mister_btn, "mapping");
            self.core_map.insert(mister_btn, index);
        }
    }

    pub fn map(&mut self, sdl_btn: u8) -> Option<u8> {
        let snes_btn = self.map[sdl_btn as usize];
        self.core_map.get(snes_btn).copied()
    }

    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }

    pub fn clear(&mut self) {
        self.bits.fill(false);
    }

    pub fn press(&mut self, button: MisterFpgaButtons) {
        if button != MisterFpgaButtons::NoMapping {
            self.bits
                .set(*self.core_map.get(button).unwrap() as usize, true);
        }
    }

    pub fn down(&mut self, sdl_btn: u8) -> u32 {
        let index = self.map(sdl_btn);
        if let Some(i) = index {
            self.bits.set(i as usize, true);
        }

        if tracing::enabled!(tracing::Level::TRACE) {
            let mask = format!("0b{:016b}", self.value());
            let snes = self.map[sdl_btn as usize];
            trace!(?sdl_btn, ?snes, ?index, ?mask, "Button down");
        }

        self.value()
    }

    pub fn up(&mut self, sdl_btn: u8) -> u32 {
        let index = self.map(sdl_btn);
        if let Some(i) = index {
            self.bits.set(i as usize, false);
        }

        if tracing::enabled!(tracing::Level::TRACE) {
            let mask = format!("0b{:016b}", self.value());
            let snes = self.map[sdl_btn as usize];
            trace!(?sdl_btn, ?snes, ?index, ?mask, "Button up");
        }

        self.value()
    }

    pub fn set(&mut self, v: u32) {
        self.bits.store(v);
    }

    pub fn value(&self) -> u32 {
        self.bits.load()
    }

    pub fn display(&self) -> String {
        let mut s = String::new();
        for (i, btn) in self.map.iter().enumerate() {
            if self.bits.get(i).as_deref() == Some(&true) {
                s.push_str(&format!("{:?} ", btn));
            }
        }
        s
    }
}
