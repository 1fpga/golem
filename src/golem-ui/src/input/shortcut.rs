use crate::input::InputState;
use bitvec::array::BitArray;
use itertools::Itertools;
use sdl3::gamepad::{Axis, Button};
use sdl3::keyboard::Scancode;
use serde::{Deserialize, Deserializer, Serialize};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::Hasher;
use std::str::FromStr;
use strum::{EnumCount, EnumString};
use tracing::error;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i8)]
pub enum AxisValue {
    /// At the end of the axis.
    HighNegative = -2,

    /// Around middle.
    LowNegative = -1,

    /// Around 0.
    #[default]
    Idle = 0,

    /// Around middle.
    LowPositive = 1,

    /// At the end of the axis.
    HighPositive = 2,
}

impl Display for AxisValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AxisValue::Idle => Ok(()),
            AxisValue::LowPositive => write!(f, ">"),
            AxisValue::HighPositive => write!(f, ">>"),
            AxisValue::LowNegative => write!(f, "<"),
            AxisValue::HighNegative => write!(f, "<<"),
        }
    }
}

impl FromStr for AxisValue {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ">" => Ok(Self::LowPositive),
            "<" => Ok(Self::LowNegative),
            ">>" => Ok(Self::HighPositive),
            "<<" => Ok(Self::HighNegative),
            "-" => Ok(Self::Idle),
            _ => Err("Invalid axis value."),
        }
    }
}

impl From<i16> for AxisValue {
    fn from(value: i16) -> Self {
        if value <= i16::MIN / 2 {
            Self::HighNegative
        } else if value <= i16::MIN / 4 {
            Self::LowNegative
        } else if value >= i16::MAX / 2 {
            Self::HighPositive
        } else if value >= i16::MAX / 4 {
            Self::LowPositive
        } else {
            Self::Idle
        }
    }
}

impl AxisValue {
    pub fn matches(&self, value: i16) -> bool {
        match self {
            AxisValue::Idle => (-32..32).contains(&value),
            AxisValue::LowPositive => ((i16::MAX / 4)..(i16::MAX / 2)).contains(&value),
            AxisValue::HighPositive => ((i16::MAX / 2)..).contains(&value),
            AxisValue::LowNegative => (i16::MIN / 2..i16::MIN / 4).rev().contains(&value),
            AxisValue::HighNegative => (..(i16::MIN / 2)).contains(&value),
        }
    }

    pub fn is_negative(&self) -> bool {
        matches!(self, Self::HighNegative | Self::LowNegative)
    }

    pub fn is_positive(&self) -> bool {
        matches!(self, Self::HighPositive | Self::LowPositive)
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }
}

/// A set of modifier keys. These are indiscriminate for whether right/left keys are pressed.
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, EnumString, strum::Display, EnumCount, strum::FromRepr,
)]
#[repr(u8)]
pub enum Modifiers {
    #[strum(ascii_case_insensitive)]
    Shift = 0,

    #[strum(ascii_case_insensitive)]
    Ctrl = 1,

    #[strum(ascii_case_insensitive)]
    Alt = 2,

    #[strum(ascii_case_insensitive)]
    Gui = 3,
}

/// A shortcut that can be compared to an input state.
/// This is normally paired with a command.
#[derive(Default, Clone, PartialEq, Eq)]
pub struct Shortcut {
    keyboard:
        BitArray<[u32; (sdl3::sys::SDL_Scancode::SDL_NUM_SCANCODES as usize) / size_of::<u32>()]>,
    keyboard_modifiers: [bool; Modifiers::COUNT],
    gamepad_button: BitArray<
        [u32; (sdl3::sys::SDL_GamepadButton::SDL_GAMEPAD_BUTTON_MAX as usize) / size_of::<u32>()],
    >,
    axis: [AxisValue; sdl3::sys::SDL_GamepadAxis::SDL_GAMEPAD_AXIS_MAX as usize],
}

impl std::fmt::Debug for Shortcut {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (modifiers, keys, buttons, axis) = self.stringify_fields();

        f.debug_struct("Shortcut")
            .field("keyboard", &keys)
            .field("keyboard_modifiers", &modifiers)
            .field("gamepad_button", &buttons)
            .field("axis", &axis)
            .finish()
    }
}

impl PartialOrd for Shortcut {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.to_string().cmp(&other.to_string()))
    }
}

impl Ord for Shortcut {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl std::hash::Hash for Shortcut {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.keyboard.hash(state);
        self.keyboard_modifiers.hash(state);
        self.gamepad_button.hash(state);
        self.axis.hash(state);
    }
}

#[inline]
fn index_to_button(i: usize) -> Button {
    Button::from_ll(unsafe { std::mem::transmute(i as i32) }).expect("Invalid button index?")
}

#[inline]
fn index_to_axis(i: usize) -> Axis {
    Axis::from_ll(unsafe { std::mem::transmute(i as i32) }).expect("Invalid axis index?")
}

impl Display for Shortcut {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (modifiers, keys, buttons, axis) = self.stringify_fields();

        [modifiers, keys, buttons, axis]
            .iter()
            .filter(|x| !x.is_empty())
            .join(" + ")
            .fmt(f)
    }
}

impl Serialize for Shortcut {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Shortcut {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

impl FromStr for Shortcut {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = Self::default();
        for shortcut in s.split('+').map(|x| x.trim()) {
            if shortcut.is_empty() {
                continue;
            }

            if (shortcut.starts_with('"') && shortcut.ends_with('"'))
                || (shortcut.starts_with('\'') && shortcut.ends_with('\''))
            {
                let s = &shortcut[1..shortcut.len() - 1];
                if let Some(key) = Scancode::from_name(s) {
                    result.add_key(key);
                } else {
                    error!(s, "Invalid key");
                    return Err("Invalid shortcut: invalid Key");
                }
            } else if let Ok(key) = Modifiers::from_str(shortcut) {
                result.add_modifier(key);
            } else if let Some(button) = Button::from_string(shortcut) {
                result.add_gamepad_button(button);
            } else if let Some((axis, value)) = shortcut
                .find(|c: char| !c.is_alphanumeric())
                .map(|i| shortcut.split_at(i))
            {
                let axis = Axis::from_string(axis.trim()).ok_or("Invalid axis name.")?;
                let value = AxisValue::from_str(value.trim_end_matches(')'))
                    .map_err(|_| "Invalid axis value.")?;
                result.add_axis(axis, value);
            } else {
                error!(shortcut, "Invalid shortcut");
                return Err("Invalid shortcut: invalid shortcut");
            }
        }

        Ok(result)
    }
}

impl Shortcut {
    fn stringify_fields(&self) -> (String, String, String, String) {
        let keys = self
            .keyboard
            .iter()
            .enumerate()
            .filter(|(_, x)| **x)
            .map(|(i, _)| {
                let k = Scancode::from_i32(i as i32).expect("Invalid scancode index?");
                format!("'{}'", k)
            })
            .join(" + ");
        let modifiers = self
            .keyboard_modifiers
            .iter()
            .enumerate()
            .filter(|(_, x)| **x)
            .map(|(i, _)| Modifiers::from_repr(i as u8).unwrap().to_string())
            .join(" + ");
        let buttons = self
            .gamepad_button
            .iter()
            .enumerate()
            .filter(|(_, x)| **x)
            .map(|(i, _)| index_to_button(i).string())
            .join(" + ");
        let axis = self
            .axis
            .iter()
            .enumerate()
            .filter(|(_, v)| !v.is_idle())
            .map(|(i, v)| format!("{}{}", index_to_axis(i).string(), v))
            .join(" + ");

        (modifiers, keys, buttons, axis)
    }

    pub fn with_key(mut self, code: Scancode) -> Self {
        self.add_key(code);
        self
    }

    pub fn add_key(&mut self, code: Scancode) -> bool {
        let i = code as usize;
        self.keyboard.set(i, true);
        true
    }

    pub fn with_modifier(mut self, modifier: Modifiers) -> Self {
        self.add_modifier(modifier);
        self
    }

    pub fn add_modifier(&mut self, modifier: Modifiers) -> bool {
        self.keyboard_modifiers[modifier as usize] = true;
        true
    }

    pub fn with_gamepad_button(mut self, button: Button) -> Self {
        self.add_gamepad_button(button);
        self
    }

    pub fn add_gamepad_button(&mut self, button: Button) -> bool {
        self.gamepad_button.set(button as usize, true);
        true
    }

    pub fn with_axis(mut self, axis: Axis, value: impl Into<AxisValue>) -> Self {
        self.add_axis(axis, value);
        self
    }

    pub fn add_axis(&mut self, axis: Axis, value: impl Into<AxisValue>) -> bool {
        let i = axis as usize;
        if i > self.axis.len() {
            return false;
        }

        let v: AxisValue = value.into();
        if v.is_idle() {
            return false;
        }

        let c = self.axis.get(i).copied().unwrap_or(AxisValue::Idle);
        if (c.is_negative() && v < c) || (c.is_positive() && v > c) || c.is_idle() {
            self.axis[i] = v;
            true
        } else {
            false
        }
    }

    pub fn matches(&self, state: &InputState) -> bool {
        self.keyboard
            .iter()
            .enumerate()
            .filter(|(_, x)| **x)
            .all(|(i, _)| {
                state
                    .keyboard
                    .contains(&Scancode::from_i32(i as i32).expect("Invalid scancode index?"))
            })
            && self
                .keyboard_modifiers
                .iter()
                .enumerate()
                .filter(|(_, x)| **x)
                .all(
                    |(i, _)| match Modifiers::from_repr(i as u8).expect("Invalid modifier?") {
                        Modifiers::Shift => {
                            state.keyboard.contains(&Scancode::LShift)
                                || state.keyboard.contains(&Scancode::RShift)
                        }
                        Modifiers::Ctrl => {
                            state.keyboard.contains(&Scancode::LCtrl)
                                || state.keyboard.contains(&Scancode::RCtrl)
                        }
                        Modifiers::Alt => {
                            state.keyboard.contains(&Scancode::LAlt)
                                || state.keyboard.contains(&Scancode::RAlt)
                        }
                        Modifiers::Gui => {
                            state.keyboard.contains(&Scancode::LGui)
                                || state.keyboard.contains(&Scancode::RGui)
                        }
                    },
                )
            && self
                .gamepad_button
                .iter()
                .enumerate()
                .filter(|(_, x)| **x)
                .all(|(i, _)| {
                    let x = index_to_button(i);
                    state.gamepads.values().any(|gp| gp.contains(&x))
                })
            && self
                .axis
                .iter()
                .enumerate()
                .filter(|(_, x)| !x.is_idle())
                .all(|(i, v)| {
                    let x = index_to_axis(i);
                    state
                        .axis
                        .values()
                        .any(|gp| gp.get(&x).map(|x| v.matches(*x)).unwrap_or(false))
                })
    }

    pub fn is_empty(&self) -> bool {
        self.keyboard.iter().all(|x| !*x)
            && self.keyboard_modifiers.iter().all(|x| !*x)
            && self.gamepad_button.iter().all(|x| !*x)
            && self.axis.iter().all(|x| x.is_idle())
    }
}

#[test]
fn shortcut_matches() {
    let mut state = InputState::default();
    state.key_down(Scancode::A);
    state.key_down(Scancode::B);
    state.controller_button_down(0, Button::DPadUp);
    state.controller_axis_motion(0, Axis::LeftX, 10000);

    assert!(Shortcut::default().with_key(Scancode::A).matches(&state));
    assert!(Shortcut::default()
        .with_key(Scancode::B)
        .with_gamepad_button(Button::DPadUp)
        .matches(&state));

    let shortcut = Shortcut::default()
        .with_key(Scancode::A)
        .with_key(Scancode::C);
    assert!(!shortcut.matches(&state));
}

#[test]
fn axis_ord() {
    assert!(AxisValue::HighNegative < AxisValue::LowNegative);
    assert!(AxisValue::HighNegative < AxisValue::LowPositive);
    assert!(AxisValue::HighPositive > AxisValue::LowNegative);
    assert!(AxisValue::HighPositive > AxisValue::LowPositive);
    assert!(AxisValue::Idle > AxisValue::LowNegative);
    assert!(AxisValue::Idle < AxisValue::LowPositive);
}
