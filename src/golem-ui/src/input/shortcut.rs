use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::Hasher;
use std::str::FromStr;

use itertools::Itertools;
use sdl3::gamepad::{Axis, Button};
use sdl3::keyboard::Scancode;
use serde::{Deserialize, Deserializer, Serialize};

use crate::input::InputState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AxisValue {
    /// At the end of the axis.
    HighPositive,

    /// Around middle.
    LowPositive,

    /// Around 0.
    Idle,

    /// Around middle.
    LowNegative,

    /// At the end of the axis.
    HighNegative,
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
            AxisValue::Idle => (-10..10).contains(&value),
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

/// A shortcut that can be compared to an input state.
/// This is normally paired with a command.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    keyboard: HashSet<Scancode>,
    gamepad_button: HashSet<Button>,
    axis: HashMap<Axis, AxisValue>,
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
        self.keyboard
            .iter()
            .sorted_by(|a, b| (**a as i32).cmp(&(**b as i32)))
            .collect::<Vec<_>>()
            .hash(state);
        self.gamepad_button
            .iter()
            .sorted_by(|a, b| (**a as i32).cmp(&(**b as i32)))
            .collect::<Vec<_>>()
            .hash(state);
        self.axis
            .iter()
            .sorted_by(|(a, _), (b, _)| (**a as i32).cmp(&(**b as i32)))
            .collect::<Vec<_>>()
            .hash(state);
    }
}

impl Display for Shortcut {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let keys = self
            .keyboard
            .iter()
            .map(|k| format!("'{}'", k.name()))
            .join(" + ");
        let buttons = self.gamepad_button.iter().map(|b| b.string()).join(" + ");
        let axis = self
            .axis
            .iter()
            .map(|(a, v)| format!("{}{}", a.string(), v))
            .join(" + ");

        [keys, buttons, axis]
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
        let deser = String::deserialize(deserializer)?;
        deser.parse().map_err(serde::de::Error::custom)
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
                    return Err("Invalid shortcut: invalid Key");
                }
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
                return Err("Invalid shortcut: invalid shortcut");
            }
        }

        Ok(result)
    }
}

impl Shortcut {
    pub fn with_key(mut self, code: Scancode) -> Self {
        self.add_key(code);
        self
    }

    pub fn add_key(&mut self, code: Scancode) -> bool {
        self.keyboard.insert(code);
        true
    }
    pub fn add_gamepad_button(&mut self, button: Button) -> bool {
        self.gamepad_button.insert(button);
        true
    }
    pub fn add_axis(&mut self, axis: Axis, value: impl Into<AxisValue>) -> bool {
        let v: AxisValue = value.into();
        if v.is_idle() {
            return false;
        }

        let c = self.axis.get(&axis).copied().unwrap_or(AxisValue::Idle);
        if (c.is_negative() && v < c) || (c.is_positive() && v > c) || c.is_idle() {
            self.axis.insert(axis, v);
            true
        } else {
            false
        }
    }

    pub fn matches(&self, state: &InputState) -> bool {
        self.keyboard.iter().all(|x| state.keyboard.contains(x))
            && self
                .gamepad_button
                .iter()
                .all(|x| state.gamepads.values().any(|gp| gp.contains(x)))
            && self.axis.iter().all(|(a, v)| {
                state
                    .axis
                    .values()
                    .any(|gp| gp.get(a).map(|x| v.matches(*x)).unwrap_or(false))
            })
    }
}

#[test]
fn shortcut_matches() {
    let mut state = InputState::default();
    state.key_down(Scancode::A);
    state.key_down(Scancode::B);
    state.controller_button_down(0, Button::DPadUp);

    let shortcut = Shortcut::default().with_key(Scancode::A);
    assert!(shortcut.matches(&state));
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
