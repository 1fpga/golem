use bitvec::prelude::*;
use itertools::Itertools;
use sdl3::keyboard::Scancode;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

pub mod commands;

#[derive(Default, Debug, Clone)]
pub struct InputState {
    pub keyboard: HashSet<Scancode>,
    pub gamepads: HashMap<u32, BitArray<[u32; 4], Lsb0>>,
    pub axis: [(Option<i32>, Option<i32>); 4],
    pub mouse: (i32, i32),
}

impl InputState {
    pub fn clear(&mut self) {
        self.keyboard.clear();
        self.gamepads.clear();
        self.axis = [(None, None); 4];
        self.mouse = (0, 0);
    }

    pub fn key_down(&mut self, code: Scancode) {
        self.keyboard.insert(code);
    }

    pub fn key_up(&mut self, code: Scancode) {
        self.keyboard.remove(&code);
    }

    pub fn gamepad_button_down(&mut self, joystick: u32, button: u8) {
        self.gamepads
            .entry(joystick)
            .or_default()
            .set(button as usize, true);
    }

    pub fn gamepad_button_up(&mut self, joystick: u32, button: u8) {
        self.gamepads
            .entry(joystick)
            .or_default()
            .set(button as usize, false);
    }

    pub fn is_empty(&self) -> bool {
        self.keyboard.is_empty()
            && self
                .gamepads
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
        }) && state.gamepads.iter().any(|(_i, gp)| {
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
