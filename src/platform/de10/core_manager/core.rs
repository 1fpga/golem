use crate::platform::Core;
use mister_fpga::config_string::{ConfigMenu, LoadFileInfo};
use mister_fpga::core::MisterFpgaCore;
use mister_fpga::types::StatusBitMap;
use sdl3::keyboard::Keycode;
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

    fn status_mask(&self) -> StatusBitMap {
        self.config().status_bit_map_mask()
    }

    fn status_bits(&self) -> StatusBitMap {
        *self.status_bits()
    }

    fn set_status_bits(&mut self, bits: StatusBitMap) {
        self.send_status_bits(bits)
    }

    fn send_key(&mut self, key: Keycode) {
        self.send_key_code(key)
    }

    fn sdl_joy_button_down(&mut self, joystick_idx: u8, button: u8) {
        self.gamepad_button_down(joystick_idx, button)
    }

    fn sdl_joy_button_up(&mut self, joystick_idx: u8, button: u8) {
        self.gamepad_button_up(joystick_idx, button)
    }
}
