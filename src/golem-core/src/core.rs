use image::DynamicImage;
use mister_fpga::config_string::{ConfigMenu, LoadFileInfo};
use mister_fpga::core::file::SdCard;
use mister_fpga::types::StatusBitMap;
use sdl3::gamepad::{Axis, Button};
use sdl3::keyboard::Scancode;
use std::path::{Path, PathBuf};

/// A core that be used in the Golem platform.
pub struct GolemCore {
    inner: mister_fpga::core::MisterFpgaCore,
    loaded_file: Option<PathBuf>,
}

impl GolemCore {
    pub fn new(inner: mister_fpga::core::MisterFpgaCore) -> Self {
        Self {
            inner,
            loaded_file: None,
        }
    }

    pub fn send_to_framebuffer(&mut self, image_raw: impl AsRef<[u8]>) -> Result<(), String> {
        self.inner.send_to_menu_framebuffer(image_raw.as_ref())?;
        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.inner.config().name
    }

    pub fn current_game(&self) -> Option<&Path> {
        self.loaded_file.as_deref()
    }

    pub fn load_file(
        &mut self,
        path: &Path,
        file_info: Option<LoadFileInfo>,
    ) -> Result<(), String> {
        let update = file_info.as_ref().map(|l| l.index == 0).unwrap_or(true);
        self.inner.load_file(path, file_info)?;
        if update {
            self.loaded_file = Some(path.to_path_buf());
        }

        Ok(())
    }

    pub fn end_send_file(&mut self) -> Result<(), String> {
        self.inner.end_send_file()
    }

    pub fn version(&self) -> Option<&str> {
        self.inner.config().version()
    }

    pub fn mount_sav(&mut self, path: &Path) -> Result<(), String> {
        self.inner.mount(SdCard::from_path(path)?, 0)?;
        Ok(())
    }

    pub fn check_sav(&mut self) -> Result<(), String> {
        while self.inner.poll_mounts()? {}
        Ok(())
    }

    pub fn menu_options(&self) -> &[ConfigMenu] {
        self.inner.config().menu.as_slice()
    }

    pub fn trigger_menu(&mut self, menu: &ConfigMenu) -> Result<bool, String> {
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

    pub fn reset(&mut self) -> Result<(), String> {
        self.inner.soft_reset();
        Ok(())
    }

    pub fn status_mask(&self) -> StatusBitMap {
        self.inner.config().status_bit_map_mask()
    }

    pub fn status_bits(&self) -> StatusBitMap {
        *self.inner.status_bits()
    }

    pub fn set_status_bits(&mut self, bits: StatusBitMap) {
        self.inner.send_status_bits(bits)
    }

    pub fn status_pulse(&mut self, bit: usize) {
        let mut bits = self.status_bits();
        bits.set(bit, true);
        self.set_status_bits(bits);

        bits.set(bit, false);
        self.set_status_bits(bits);
    }

    pub fn take_screenshot(&mut self) -> Result<DynamicImage, String> {
        self.inner.take_screenshot()
    }

    pub fn key_down(&mut self, key: Scancode) {
        self.inner.key_down(key)
    }

    pub fn key_up(&mut self, key: Scancode) {
        self.inner.key_up(key)
    }

    pub fn sdl_button_down(&mut self, controller: u8, button: Button) {
        self.inner.gamepad_button_down(controller, button as u8)
    }

    pub fn sdl_button_up(&mut self, controller: u8, button: Button) {
        self.inner.gamepad_button_up(controller, button as u8)
    }

    pub fn sdl_axis_motion(&mut self, _controller: u8, _axis: Axis, _value: i16) {
        // TODO: do this.
    }

    pub fn save_states(&mut self) -> Option<&mut [mister_fpga::savestate::SaveState]> {
        self.inner
            .save_states_mut()
            .map(|manager| manager.slots_mut())
    }
}
