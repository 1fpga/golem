use crate::platform::Core;
use image::DynamicImage;
use mister_fpga::config_string::{ConfigMenu, LoadFileInfo};
use mister_fpga::core::MisterFpgaCore;
use mister_fpga::types::StatusBitMap;
use sdl3::gamepad::{Axis, Button};
use sdl3::keyboard::Scancode;
use std::path::Path;

impl Core for MisterFpgaCore {
    fn name(&self) -> &str {
        &self.config().name
    }

    fn load_file(&mut self, path: &Path, file_info: Option<LoadFileInfo>) -> Result<(), String> {
        self.send_file(path, file_info)
    }

    fn version(&self) -> Option<&str> {
        self.config().version()
    }

    fn menu_options(&self) -> &[ConfigMenu] {
        self.config().menu.as_slice()
    }

    fn trigger_menu(&mut self, menu: &ConfigMenu) -> Result<bool, String> {
        match menu {
            ConfigMenu::HideIf(cond, sub) | ConfigMenu::DisableIf(cond, sub) => {
                if self.config().status_bit_map_mask().get(*cond as usize) {
                    self.trigger_menu(sub)
                } else {
                    Err("Cannot trigger menu".to_string())
                }
            }
            ConfigMenu::HideUnless(cond, sub) | ConfigMenu::DisableUnless(cond, sub) => {
                if !self.config().status_bit_map_mask().get(*cond as usize) {
                    self.trigger_menu(sub)
                } else {
                    Err("Cannot trigger menu".to_string())
                }
            }
            ConfigMenu::Option { bits, choices, .. } => {
                let (from, to) = (bits.start, bits.end);
                let mut bits = *self.status_bits();
                let max = choices.len();
                let value = bits.get_range(from..to) as usize;
                bits.set_range(from..to, ((value + 1) % max) as u32);
                self.set_status_bits(bits);
                Ok(true)
            }
            ConfigMenu::Trigger { index, .. } => {
                self.status_pulse(*index as usize);
                Ok(true)
            }
            ConfigMenu::PageItem(_, sub) => self.trigger_menu(sub),

            // TODO: see if we can implement more (like Load File).
            _ => Ok(false),
        }
    }

    fn status_mask(&self) -> StatusBitMap {
        self.config().status_bit_map_mask()
    }

    fn status_bits(&self) -> StatusBitMap {
        *self.status_bits()
    }

    fn set_status_bits(&mut self, bits: StatusBitMap) {
        self.send_status_bits(bits)
    }

    fn take_screenshot(&mut self) -> Result<DynamicImage, String> {
        self.take_screenshot()
    }

    fn send_key(&mut self, key: Scancode) {
        self.send_key_code(key as u8)
    }

    fn sdl_button_down(&mut self, controller: u8, button: Button) {
        self.gamepad_button_down(controller, button as u8)
    }

    fn sdl_button_up(&mut self, controller: u8, button: Button) {
        self.gamepad_button_up(controller, button as u8)
    }

    fn sdl_axis_motion(&mut self, _controller: u8, _axis: Axis, _value: i16) {
        // TODO: do this.
    }
}
