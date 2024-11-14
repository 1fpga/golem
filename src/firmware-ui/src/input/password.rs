use crate::input::shortcut::AxisValue;
use sdl3::gamepad::{Axis, Button};
use sdl3::keyboard::Scancode;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum PasswordInput {
    Keyboard(Scancode),
    GamepadButton(Button),
    GamepadAxis(Axis, AxisValue),
}

impl Display for PasswordInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordInput::Keyboard(k) => write!(f, "'{}'", k.name()),
            PasswordInput::GamepadButton(b) => write!(f, "{}", b.string()),
            PasswordInput::GamepadAxis(a, v) => write!(f, "{}{}", a.string(), v),
        }
    }
}

/// A password input. A series of button/key presses that must be entered in order to
/// unlock something. This does not store any controller information and can match
/// regardless of which controller was used.
#[derive(Debug, Clone, PartialEq)]
pub struct InputPassword(Vec<PasswordInput>);

impl Display for InputPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

impl InputPassword {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add_key(&mut self, key: Scancode) {
        self.0.push(PasswordInput::Keyboard(key));
    }

    pub fn add_controller_button(&mut self, button: Button) {
        self.0.push(PasswordInput::GamepadButton(button));
    }

    pub fn add_controller_axis(&mut self, axis: Axis, value: i16) {
        self.0
            .push(PasswordInput::GamepadAxis(axis, AxisValue::from(value)));
    }

    pub fn iter(&self) -> impl Iterator<Item = &PasswordInput> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
