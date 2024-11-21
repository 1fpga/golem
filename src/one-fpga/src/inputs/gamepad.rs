use serde::{Deserialize, Serialize};
use static_assertions::const_assert;
use strum::{Display, EnumCount, EnumIter, EnumString, FromRepr, IntoEnumIterator};

/// Gamepad buttons.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    EnumIter,
    EnumString,
    Display,
    EnumCount,
    FromRepr,
)]
#[repr(u8)]
pub enum Button {
    A = sdl3::gamepad::Button::East as u8,
    B = sdl3::gamepad::Button::South as u8,
    X = sdl3::gamepad::Button::North as u8,
    Y = sdl3::gamepad::Button::West as u8,
    Back = sdl3::gamepad::Button::Back as u8,
    Guide = sdl3::gamepad::Button::Guide as u8,
    Start = sdl3::gamepad::Button::Start as u8,
    LeftStick = sdl3::gamepad::Button::LeftStick as u8,
    RightStick = sdl3::gamepad::Button::RightStick as u8,
    LeftShoulder = sdl3::gamepad::Button::LeftShoulder as u8,
    RightShoulder = sdl3::gamepad::Button::RightShoulder as u8,
    DPadUp = sdl3::gamepad::Button::DPadUp as u8,
    DPadDown = sdl3::gamepad::Button::DPadDown as u8,
    DPadLeft = sdl3::gamepad::Button::DPadLeft as u8,
    DPadRight = sdl3::gamepad::Button::DPadRight as u8,
    Misc1 = sdl3::gamepad::Button::Misc1 as u8,
    RightPaddle1 = sdl3::gamepad::Button::RightPaddle1 as u8,
    LeftPaddle1 = sdl3::gamepad::Button::LeftPaddle1 as u8,
    RightPaddle2 = sdl3::gamepad::Button::RightPaddle2 as u8,
    LeftPaddle2 = sdl3::gamepad::Button::LeftPaddle2 as u8,
    Touchpad = sdl3::gamepad::Button::Touchpad as u8,
}

impl Button {
    pub fn as_sdl(&self) -> sdl3::gamepad::Button {
        sdl3::gamepad::Button::from(*self)
    }

    pub fn as_repr(&self) -> u8 {
        *self as u8
    }

    /// Gets a string representation of the button.
    pub fn name(&self) -> &'static str {
        match self {
            Button::A => "A",
            Button::B => "B",
            Button::X => "X",
            Button::Y => "Y",
            Button::Back => "Back",
            Button::Guide => "Guide",
            Button::Start => "Start",
            Button::LeftStick => "LeftStick",
            Button::RightStick => "RightStick",
            Button::LeftShoulder => "LeftShoulder",
            Button::RightShoulder => "RightShoulder",
            Button::DPadUp => "DPadUp",
            Button::DPadDown => "DPadDown",
            Button::DPadLeft => "DPadLeft",
            Button::DPadRight => "DPadRight",
            Button::Misc1 => "Misc1",
            Button::RightPaddle1 => "RightPaddle1",
            Button::LeftPaddle1 => "LeftPaddle1",
            Button::RightPaddle2 => "RightPaddle2",
            Button::LeftPaddle2 => "LeftPaddle2",
            Button::Touchpad => "Touchpad",
        }
    }
}

impl From<sdl3::gamepad::Button> for Button {
    fn from(button: sdl3::gamepad::Button) -> Self {
        match button {
            sdl3::gamepad::Button::East => Button::A,
            sdl3::gamepad::Button::South => Button::B,
            sdl3::gamepad::Button::North => Button::X,
            sdl3::gamepad::Button::West => Button::Y,
            sdl3::gamepad::Button::Back => Button::Back,
            sdl3::gamepad::Button::Guide => Button::Guide,
            sdl3::gamepad::Button::Start => Button::Start,
            sdl3::gamepad::Button::LeftStick => Button::LeftStick,
            sdl3::gamepad::Button::RightStick => Button::RightStick,
            sdl3::gamepad::Button::LeftShoulder => Button::LeftShoulder,
            sdl3::gamepad::Button::RightShoulder => Button::RightShoulder,
            sdl3::gamepad::Button::DPadUp => Button::DPadUp,
            sdl3::gamepad::Button::DPadDown => Button::DPadDown,
            sdl3::gamepad::Button::DPadLeft => Button::DPadLeft,
            sdl3::gamepad::Button::DPadRight => Button::DPadRight,
            sdl3::gamepad::Button::Misc1 => Button::Misc1,
            sdl3::gamepad::Button::RightPaddle1 => Button::RightPaddle1,
            sdl3::gamepad::Button::LeftPaddle1 => Button::LeftPaddle1,
            sdl3::gamepad::Button::RightPaddle2 => Button::RightPaddle2,
            sdl3::gamepad::Button::LeftPaddle2 => Button::LeftPaddle2,
            sdl3::gamepad::Button::Touchpad => Button::Touchpad,
        }
    }
}

impl From<Button> for sdl3::gamepad::Button {
    fn from(button: Button) -> Self {
        button.as_sdl()
    }
}

/// A set of pressed buttons.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct ButtonSet(u32);

// Make sure we don't have more than 32 buttons (or u32 might overflow).
const_assert!(Button::COUNT <= 32);

impl ButtonSet {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn contains(&self, button: Button) -> bool {
        self.0 & (1 << button.as_repr()) != 0
    }

    pub fn insert(&mut self, button: Button) {
        self.0 |= 1 << button.as_repr();
    }

    pub fn remove(&mut self, button: Button) {
        self.0 &= !(1 << button.as_repr());
    }
}

impl std::fmt::Debug for ButtonSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_tuple("ButtonSet");

        for b in Button::iter() {
            if self.contains(b) {
                f.field(&b.to_string());
            }
        }

        f.finish()
    }
}

/// Gamepad axes. We reuse the SDL3 type for simplicity, as it is well maintained and complete.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Axis(sdl3::gamepad::Axis);

impl Axis {
    /// Get a string representation of the axis.
    pub fn name(&self) -> String {
        self.0.string()
    }
}

impl From<sdl3::gamepad::Axis> for Axis {
    fn from(axis: sdl3::gamepad::Axis) -> Self {
        Self(axis)
    }
}

impl From<Axis> for sdl3::gamepad::Axis {
    fn from(axis: Axis) -> Self {
        axis.0
    }
}
