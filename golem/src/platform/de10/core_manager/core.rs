use crate::platform::Core;
use image::DynamicImage;
use mister_fpga::config_string::{ConfigMenu, LoadFileInfo};
use mister_fpga::types::StatusBitMap;
use sdl3::gamepad::{Axis, Button};
use sdl3::keyboard::Scancode;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct MisterFpgaCore {
    inner: mister_fpga::core::MisterFpgaCore,
    loaded_file: Option<PathBuf>,
}

impl MisterFpgaCore {
    pub fn new(inner: mister_fpga::core::MisterFpgaCore) -> Self {
        Self {
            inner,
            loaded_file: None,
        }
    }
}

impl crate::platform::SaveState for mister_fpga::savestate::SaveState {
    fn is_dirty(&self) -> bool {
        self.is_dirty()
    }

    fn write_to(&mut self, writer: impl std::io::Write) -> Result<(), String> {
        self.write_to(writer).map_err(|e| e.to_string())
    }

    fn read_from(&mut self, reader: impl Read) -> Result<(), String> {
        self.read_from(reader).map_err(|e| e.to_string())
    }
}

impl Core for MisterFpgaCore {
    type SaveState = mister_fpga::savestate::SaveState;

    fn name(&self) -> &str {
        &self.inner.config().name
    }

    fn current_game(&self) -> Option<&Path> {
        self.loaded_file.as_deref()
    }

    fn load_file(&mut self, path: &Path, file_info: Option<LoadFileInfo>) -> Result<(), String> {
        let update = file_info.as_ref().map(|l| l.index == 0).unwrap_or(true);
        self.inner.send_file(path, file_info)?;
        if update {
            self.loaded_file = Some(path.to_path_buf());
        }

        Ok(())
    }

    fn version(&self) -> Option<&str> {
        self.inner.config().version()
    }

    fn menu_options(&self) -> &[ConfigMenu] {
        self.inner.config().menu.as_slice()
    }

    fn trigger_menu(&mut self, menu: &ConfigMenu) -> Result<bool, String> {
        match menu {
            ConfigMenu::HideIf(cond, sub) | ConfigMenu::DisableIf(cond, sub) => {
                if self
                    .inner
                    .config()
                    .status_bit_map_mask()
                    .get(*cond as usize)
                {
                    self.trigger_menu(sub)
                } else {
                    Err("Cannot trigger menu".to_string())
                }
            }
            ConfigMenu::HideUnless(cond, sub) | ConfigMenu::DisableUnless(cond, sub) => {
                if !self
                    .inner
                    .config()
                    .status_bit_map_mask()
                    .get(*cond as usize)
                {
                    self.trigger_menu(sub)
                } else {
                    Err("Cannot trigger menu".to_string())
                }
            }
            ConfigMenu::Option { bits, choices, .. } => {
                let (from, to) = (bits.start, bits.end);
                let mut bits = self.status_bits();
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
        self.inner.config().status_bit_map_mask()
    }

    fn status_bits(&self) -> StatusBitMap {
        *self.inner.status_bits()
    }

    fn set_status_bits(&mut self, bits: StatusBitMap) {
        self.inner.send_status_bits(bits)
    }

    fn take_screenshot(&mut self) -> Result<DynamicImage, String> {
        self.inner.take_screenshot()
    }

    fn key_down(&mut self, key: Scancode) {
        self.inner.key_down(key)
    }

    fn key_up(&mut self, key: Scancode) {
        self.inner.key_up(key)
    }

    fn sdl_button_down(&mut self, controller: u8, button: Button) {
        self.inner.gamepad_button_down(controller, button as u8)
    }

    fn sdl_button_up(&mut self, controller: u8, button: Button) {
        self.inner.gamepad_button_up(controller, button as u8)
    }

    fn sdl_axis_motion(&mut self, _controller: u8, _axis: Axis, _value: i16) {
        // TODO: do this.
    }

    fn save_states(&mut self) -> Option<&mut [Self::SaveState]> {
        self.inner
            .save_states_mut()
            .map(|manager| manager.as_slice_mut())
    }
}
