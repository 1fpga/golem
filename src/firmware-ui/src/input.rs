use crate::input::shortcut::AxisValue;
use itertools::Itertools;
use one_fpga::inputs::{Axis, Button, Scancode};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

pub mod commands;
pub mod password;
pub mod shortcut;

/// The current status of all inputs.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct InputState {
    pub keyboard: HashSet<Scancode>,
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
            .map(|k| format!("'{}'", k.name()))
            .join(" + ");
        let controllers = self
            .gamepads
            .iter()
            .map(|(i, b)| {
                let buttons = b.iter().map(|b| b.name()).join(", ");
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
                    .map(|(a, v)| format!("{}: {}", a.name(), v))
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

        [keys, controllers, axis, mice]
            .iter()
            .filter(|x| !x.is_empty())
            .join("\n")
    }

    pub fn clear(&mut self) {
        self.keyboard.clear();
        self.gamepads.clear();
        self.axis.clear();
        self.mice.clear();
    }

    #[inline]
    pub fn key(&self, code: impl Into<Scancode>) -> bool {
        self.keyboard.contains(&code.into())
    }

    #[inline]
    pub fn key_down(&mut self, code: impl Into<Scancode>) {
        self.keyboard.insert(code.into());
    }

    #[inline]
    pub fn key_up(&mut self, code: impl Into<Scancode>) {
        self.keyboard.remove(&code.into());
    }

    #[inline]
    pub fn controller_button_down(&mut self, controller: u32, button: impl Into<Button>) {
        self.gamepads
            .entry(controller)
            .or_default()
            .insert(button.into());
    }

    #[inline]
    pub fn controller_button_up(&mut self, controller: u32, button: impl Into<Button>) {
        self.gamepads
            .entry(controller)
            .or_default()
            .remove(&button.into());
    }

    #[inline]
    pub fn controller_axis_motion(&mut self, controller: u32, axis: impl Into<Axis>, value: i16) {
        let axis = axis.into();
        if (-10..10).contains(&value) {
            self.axis.entry(controller).or_default().remove(&axis);
        } else {
            self.axis.entry(controller).or_default().insert(axis, value);
        }
    }

    pub fn mouse_move(&mut self, mouse: u32, x: i32, y: i32) {
        let m = self.mice.entry(mouse).or_default();
        m.0 += x;
        m.1 += y;
    }

    pub fn is_empty(&self) -> bool {
        self.keyboard.is_empty()
            && self.gamepads.values().all(|x| x.is_empty())
            && self
                .axis
                .values()
                .all(|x| x.values().all(|x| AxisValue::from(*x) == AxisValue::Idle))
    }

    /// Returns the number of inputs in this state.
    pub fn len(&self) -> usize {
        self.keyboard.len()
            + self.gamepads.values().map(|x| x.len()).sum::<usize>()
            + self.axis.values().map(|x| x.len()).sum::<usize>()
            + self.mice.len()
    }
}
