use bitvec::prelude::*;
use itertools::Itertools;
use sdl3::gamepad::{Axis, Button};
use sdl3::keyboard::Scancode;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::Hasher;
use std::str::FromStr;

pub mod commands;

/// The current status of all inputs.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct InputState {
    pub keyboard: HashSet<Scancode>,
    pub joysticks: HashMap<u32, BitArray<[u32; 4], Lsb0>>,
    pub gamepads: HashMap<u32, HashSet<Button>>,
    pub axis: HashMap<u32, HashMap<Axis, i16>>,
    pub mice: HashMap<u32, (i32, i32)>,
}

impl Display for InputState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_pretty())
    }
}

impl InputState {
    pub fn to_string_pretty(&self) -> String {
        let keys = self
            .keyboard
            .iter()
            .map(|k| format!(r#""{}""#, k.name()))
            .join(" + ");
        let joysticks = self
            .joysticks
            .iter()
            .map(|(i, b)| {
                let buttons = b
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| *b == true)
                    .map(|(i, _)| i.to_string())
                    .join(", ");
                if !buttons.is_empty() {
                    format!("Joystick {i}: {}", buttons)
                } else {
                    "".to_string()
                }
            })
            .filter(|x| !x.is_empty())
            .join("\n");
        let controllers = self
            .gamepads
            .iter()
            .map(|(i, b)| {
                let buttons = b.iter().map(|b| b.string()).join(", ");
                if !buttons.is_empty() {
                    format!("Gamepad {i}: {}", buttons)
                } else {
                    "".to_string()
                }
            })
            .filter(|x| !x.is_empty())
            .join("\n");
        let axis = self
            .axis
            .iter()
            .map(|(i, a)| {
                let axis = a
                    .iter()
                    .map(|(a, v)| format!("{}: {}", a.string(), v))
                    .join(", ");
                if !axis.is_empty() {
                    format!("Axis {i}: {}", axis)
                } else {
                    "".to_string()
                }
            })
            .filter(|x| !x.is_empty())
            .join("\n");
        let mice = self
            .mice
            .iter()
            .map(|(i, (x, y))| format!("Mouse {}: {}, {}", i, x, y))
            .join("\n");

        [keys, joysticks, controllers, axis, mice]
            .iter()
            .filter(|x| !x.is_empty())
            .join("\n")
    }

    pub fn clear(&mut self) {
        self.keyboard.clear();
        self.joysticks.clear();
        self.gamepads.clear();
        self.axis.clear();
        self.mice.clear();
    }

    pub fn key_down(&mut self, code: Scancode) {
        self.keyboard.insert(code);
    }

    pub fn key_up(&mut self, code: Scancode) {
        self.keyboard.remove(&code);
    }

    pub fn joystick_button_down(&mut self, joystick: u32, button: u8) {
        self.joysticks
            .entry(joystick)
            .or_default()
            .set(button as usize, true);
    }

    pub fn joystick_button_up(&mut self, joystick: u32, button: u8) {
        self.joysticks
            .entry(joystick)
            .or_default()
            .set(button as usize, false);
    }

    pub fn controller_button_down(&mut self, controller: u32, button: Button) {
        self.gamepads.entry(controller).or_default().insert(button);
    }

    pub fn controller_button_up(&mut self, controller: u32, button: Button) {
        self.gamepads.entry(controller).or_default().remove(&button);
    }

    pub fn controller_axis_motion(&mut self, controller: u32, axis: Axis, value: i16) {
        if value >= -10 && value < 10 {
            self.axis.entry(controller).or_default().remove(&axis);
        } else {
            self.axis.entry(controller).or_default().insert(axis, value);
        }
    }

    pub fn mouse_move(&mut self, mouse: u32, x: i32, y: i32) {
        let mut m = self.mice.entry(mouse).or_default();
        m.0 += x;
        m.1 += y;
    }

    pub fn is_empty(&self) -> bool {
        self.keyboard.is_empty()
            && self
                .joysticks
                .values()
                .all(|gp| gp.as_raw_slice().iter().all(|b| *b == 0))
            && self.gamepads.values().all(|x| x.is_empty())
            && self
                .axis
                .values()
                .all(|x| x.values().all(|x| AxisValue::from(*x) == AxisValue::Idle))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
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
            AxisValue::LowPositive => write!(f, "+Mid"),
            AxisValue::HighPositive => write!(f, "+End"),
            AxisValue::LowNegative => write!(f, "-Mid"),
            AxisValue::HighNegative => write!(f, "-End"),
        }
    }
}

impl FromStr for AxisValue {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+Mid" => Ok(Self::LowPositive),
            "+End" => Ok(Self::HighPositive),
            "-Mid" => Ok(Self::LowNegative),
            "-End" => Ok(Self::HighNegative),
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
            AxisValue::Idle => value >= -10 && value < 10,
            AxisValue::LowPositive => value >= i16::MAX / 4 && value < i16::MAX / 2,
            AxisValue::HighPositive => value >= i16::MAX / 2,
            AxisValue::LowNegative => value <= i16::MIN / 4 && value > i16::MIN / 2,
            AxisValue::HighNegative => value <= i16::MIN / 2,
        }
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
            .map(|k| format!(r#""{}""#, k.name()))
            .join(" + ");
        let buttons = self.gamepad_button.iter().map(|b| b.string()).join(" + ");
        let axis = self
            .axis
            .iter()
            .map(|(a, v)| format!("{}: {}", a.string(), v))
            .join(" + ");

        if !keys.is_empty() {
            write!(f, "{}\n", keys)?;
        }
        if !buttons.is_empty() {
            write!(f, "{}\n", buttons)?;
        }
        if !axis.is_empty() {
            write!(f, "{}\n", axis)?;
        }
        Ok(())
    }
}

impl Serialize for Shortcut {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let keys = self.keyboard.iter().map(|k| k.name()).collect::<Vec<_>>();
        let buttons = self
            .gamepad_button
            .iter()
            .map(|b| b.string())
            .collect::<Vec<_>>();
        let axis = self
            .axis
            .iter()
            .map(|(a, v)| format!("{}: {}", a.string(), v))
            .collect::<Vec<_>>();

        let mut map = serializer.serialize_map(Some(3))?;
        if !keys.is_empty() {
            map.serialize_entry("keys", &keys)?;
        }
        if !buttons.is_empty() {
            map.serialize_entry("buttons", &buttons)?;
        }
        if !axis.is_empty() {
            map.serialize_entry("axis", &axis)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Shortcut {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut result = Self::default();
        let deser = BTreeMap::<String, Vec<String>>::deserialize(deserializer)?;
        let keys = deser.get("keys");
        if let Some(keys) = keys {
            for key in keys {
                result.add_key(
                    Scancode::from_name(&key)
                        .ok_or_else(|| serde::de::Error::custom("Invalid key name."))?,
                );
            }
        }

        let buttons = deser.get("buttons");
        if let Some(buttons) = buttons {
            for button in buttons {
                result.add_gamepad_button(
                    Button::from_string(button)
                        .ok_or_else(|| serde::de::Error::custom("Invalid button name."))?,
                );
            }
        }

        let axis = deser.get("axis");
        if let Some(axis) = axis {
            for axis in axis {
                let mut split = axis.split(":");
                let axis = split
                    .next()
                    .ok_or_else(|| serde::de::Error::custom("No separator."))?;
                let value = split
                    .next()
                    .ok_or_else(|| serde::de::Error::custom("Invalid axis value."))?;
                let axis = Axis::from_string(axis)
                    .ok_or_else(|| serde::de::Error::custom("Invalid axis name."))?;
                let value = AxisValue::from_str(value).map_err(serde::de::Error::custom)?;
                result.axis.insert(axis, value);
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

    pub fn add_key(&mut self, code: Scancode) {
        self.keyboard.insert(code);
    }
    pub fn add_gamepad_button(&mut self, button: Button) {
        self.gamepad_button.insert(button);
    }
    pub fn add_axis(&mut self, axis: Axis, value: AxisValue) {
        self.axis.insert(axis, value);
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
