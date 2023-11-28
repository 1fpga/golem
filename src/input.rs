use bitvec::prelude::*;
use itertools::Itertools;
use sdl3::gamepad::{Axis, Button};
use sdl3::keyboard::Scancode;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

pub mod commands;

#[derive(Default, Debug, Clone)]
pub struct InputState {
    pub keyboard: HashSet<Scancode>,
    pub joysticks: HashMap<u32, BitArray<[u32; 4], Lsb0>>,
    pub gamepads: HashMap<u32, HashSet<Button>>,
    pub axis: HashMap<u32, HashMap<Axis, i16>>,
    pub mouse: (i32, i32),
}

impl Display for InputState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

        write!(
            f,
            "Keys: {}\n{}\n{}\n{}",
            keys, joysticks, controllers, axis
        )
    }
}

impl InputState {
    pub fn clear(&mut self) {
        self.keyboard.clear();
        self.joysticks.clear();
        self.axis.clear();
        self.mouse = (0, 0);
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
        self.axis.entry(controller).or_default().insert(axis, value);
    }

    pub fn is_empty(&self) -> bool {
        self.keyboard.is_empty()
            && self
                .joysticks
                .values()
                .all(|gp| gp.as_raw_slice().iter().all(|b| *b == 0))
    }
}

#[derive(Default, Debug, Clone, Hash)]
pub struct BasicInputShortcut {
    keyboard: [Option<Scancode>; 8],
    gamepad_button: BitArray<[u32; 8], Lsb0>,
    axis: [(Option<u8>, Option<i32>); 4],
}

impl Display for BasicInputShortcut {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let keys = self
            .keyboard
            .iter()
            .filter_map(|k| k.map(|k| format!(r#""{}""#, k.name())))
            .join(" + ");
        let buttons = self
            .gamepad_button
            .iter()
            .enumerate()
            .filter_map(|(i, b)| if *b { Some(i) } else { None })
            .map(|b| format!("Button {}", b))
            .join(" + ");

        if keys == "" {
            write!(f, "{}", buttons)
        } else if buttons == "" {
            write!(f, "{}", keys)
        } else {
            write!(f, "{}, {}", keys, buttons)
        }
    }
}

impl Serialize for BasicInputShortcut {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let keys = self
            .keyboard
            .iter()
            .filter_map(|k| k.map(|k| k.name()))
            .collect::<Vec<_>>();
        let buttons = self
            .gamepad_button
            .iter()
            .enumerate()
            .filter_map(|(i, b)| if *b { Some(i) } else { None })
            .collect::<Vec<_>>();

        let mut seq = serializer.serialize_seq(Some(keys.len() + buttons.len()))?;
        for key in keys {
            seq.serialize_element(&key)?;
        }
        for button in buttons {
            seq.serialize_element(&format!("button {}", button))?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for BasicInputShortcut {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut result = Self::default();
        let deser = Vec::<String>::deserialize(deserializer)?;
        for key in deser {
            if let Some(b) = key.strip_prefix("button ") {
                result.add_gamepad_button(b.parse().map_err(serde::de::Error::custom)?);
            } else {
                result.add_key(
                    Scancode::from_name(&key)
                        .ok_or_else(|| serde::de::Error::custom("Invalid key name."))?,
                );
            }
        }
        Ok(result)
    }
}

impl BasicInputShortcut {
    pub fn with_key(mut self, code: Scancode) -> Self {
        self.add_key(code);
        self
    }

    pub fn add_key(&mut self, code: Scancode) {
        if self.keyboard.iter().any(|k| k == &Some(code)) {
            return;
        }
        self.keyboard
            .iter_mut()
            .find_or_last(|k| k.is_none())
            .map(|k| *k = Some(code));
    }
    pub fn add_gamepad_button(&mut self, button: u8) {
        self.gamepad_button.set(button as usize, true);
    }

    pub fn matches(&self, state: &InputState) -> bool {
        if self.keyboard.iter().all(|k| {
            if let Some(k) = k {
                state.keyboard.contains(k)
            } else {
                true
            }
        }) && state.joysticks.iter().any(|(_i, gp)| {
            gp.iter()
                .zip(self.gamepad_button.iter())
                .all(|(a, b)| a == b)
        }) {
            true
        } else {
            false
        }
    }
}
