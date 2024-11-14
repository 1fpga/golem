use std::any::Any;
use std::time::SystemTime;

use image::{ColorType, DynamicImage};

use crate::core::{Bios, CoreSettings, Error, MountedFile, Rom, SaveState, SettingId};
use crate::inputs::gamepad::ButtonSet;
use crate::inputs::keyboard::ScancodeSet;
use crate::inputs::{Button, Scancode};
use crate::Core;

/// A Core that does nothing.
pub struct NullCore;

impl Core for NullCore {
    fn init(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn name(&self) -> &str {
        "null"
    }

    fn reset(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn set_volume(&mut self, _volume: u8) -> Result<(), Error> {
        Ok(())
    }

    fn set_rtc(&mut self, _time: SystemTime) -> Result<(), Error> {
        Ok(())
    }

    fn screenshot(&self) -> Result<DynamicImage, Error> {
        Ok(DynamicImage::new(1, 1, ColorType::Rgb8))
    }

    fn save_state_mut(&mut self, _slot: usize) -> Result<Option<&mut dyn SaveState>, Error> {
        Ok(None)
    }

    fn save_state(&self, _slot: usize) -> Result<Option<&dyn SaveState>, Error> {
        Ok(None)
    }

    fn mounted_file_mut(&mut self, _slot: usize) -> Result<Option<&mut dyn MountedFile>, Error> {
        Ok(None)
    }

    fn send_rom(&mut self, _rom: Rom) -> Result<(), Error> {
        Ok(())
    }

    fn send_bios(&mut self, _bios: Bios) -> Result<(), Error> {
        Ok(())
    }

    fn key_up(&mut self, _key: Scancode) -> Result<(), Error> {
        Ok(())
    }

    fn key_down(&mut self, _key: Scancode) -> Result<(), Error> {
        Ok(())
    }

    fn keys_set(&mut self, _keys: ScancodeSet) -> Result<(), Error> {
        Ok(())
    }

    fn keys(&self) -> Result<ScancodeSet, Error> {
        Ok(ScancodeSet::new())
    }

    fn gamepad_button_up(&mut self, _index: usize, _button: Button) -> Result<(), Error> {
        Ok(())
    }

    fn gamepad_button_down(&mut self, _index: usize, _button: Button) -> Result<(), Error> {
        Ok(())
    }

    fn gamepad_buttons_set(&mut self, _index: usize, _buttons: ButtonSet) -> Result<(), Error> {
        Ok(())
    }

    fn gamepad_buttons(&self, _index: usize) -> Result<Option<ButtonSet>, Error> {
        Ok(None)
    }

    fn settings(&self) -> Result<CoreSettings, Error> {
        // TODO: add some basic items.
        Ok(CoreSettings::new("null".to_string(), vec![]))
    }

    fn trigger(&mut self, _id: SettingId) -> Result<(), Error> {
        Ok(())
    }

    fn file_select(&mut self, _id: SettingId, _path: String) -> Result<(), Error> {
        Ok(())
    }

    fn int_option(&mut self, _id: SettingId, _value: u32) -> Result<u32, Error> {
        Ok(0)
    }

    fn bool_option(&mut self, _id: SettingId, _value: bool) -> Result<bool, Error> {
        Ok(false)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn quit(&mut self) {}

    fn should_quit(&self) -> bool {
        false
    }
}
