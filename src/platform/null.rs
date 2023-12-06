use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::platform::sdl::SdlPlatform;
use crate::platform::GoLEmPlatform;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::BinaryColor;
use image::DynamicImage;
use mister_fpga::config_string::{ConfigMenu, LoadFileInfo};
use mister_fpga::types::StatusBitMap;
use sdl3::event::Event;
use sdl3::gamepad::{Axis, Button};
use sdl3::keyboard::Scancode;
use std::path::Path;

pub struct NullCore;

impl super::Core for NullCore {
    fn name(&self) -> &str {
        ""
    }

    fn load_file(&mut self, _path: &Path, _file_info: Option<LoadFileInfo>) -> Result<(), String> {
        unreachable!()
    }

    fn version(&self) -> Option<&str> {
        unreachable!()
    }

    fn menu_options(&self) -> &[ConfigMenu] {
        unreachable!()
    }

    fn trigger_menu(&mut self, _menu: &ConfigMenu) -> Result<bool, String> {
        unreachable!()
    }

    fn status_mask(&self) -> StatusBitMap {
        unreachable!()
    }

    fn status_bits(&self) -> StatusBitMap {
        unreachable!()
    }

    fn set_status_bits(&mut self, _bits: StatusBitMap) {
        unreachable!()
    }

    fn take_screenshot(&mut self) -> Result<DynamicImage, String> {
        unreachable!()
    }

    fn send_key(&mut self, _key: Scancode) {
        unreachable!()
    }

    fn sdl_button_down(&mut self, _joystick_idx: u8, _button: Button) {
        unreachable!()
    }

    fn sdl_button_up(&mut self, _joystick_idx: u8, _button: Button) {
        unreachable!()
    }

    fn sdl_axis_motion(&mut self, controller: u8, axis: Axis, value: i16) {
        unreachable!()
    }
}

pub struct NullCoreManager;

impl super::CoreManager for NullCoreManager {
    type Core = NullCore;

    fn load_program(&mut self, _path: impl AsRef<Path>) -> Result<Self::Core, String> {
        unreachable!("Platform should never run in NULL.")
    }

    fn load_menu(&mut self) -> Result<Self::Core, String> {
        unreachable!("Platform should never run in NULL.")
    }

    fn show_menu(&mut self) {
        unreachable!()
    }

    fn hide_menu(&mut self) {
        unreachable!()
    }
}

#[derive(Default)]
pub struct NullPlatform;

impl GoLEmPlatform for NullPlatform {
    type Color = BinaryColor;
    type CoreManager = NullCoreManager;

    fn init(&mut self, _flags: &crate::main_inner::Flags) {
        unreachable!("Platform should never run in NULL.")
    }

    fn update_toolbar(&mut self, _buffer: &DrawBuffer<Self::Color>) {
        unreachable!("Platform should never run in NULL.")
    }
    fn update_main(&mut self, _buffer: &DrawBuffer<Self::Color>) {
        unreachable!("Platform should never run in NULL.")
    }

    fn toolbar_dimensions(&self) -> Size {
        unreachable!("Platform should never run in NULL.")
    }
    fn main_dimensions(&self) -> Size {
        unreachable!("Platform should never run in NULL.")
    }

    fn events(&mut self) -> Vec<Event> {
        unreachable!("Platform should never run in NULL.")
    }

    fn sdl(&mut self) -> &mut SdlPlatform<Self::Color> {
        unreachable!()
    }

    fn start_loop(&mut self) {}
    fn end_loop(&mut self) {}

    fn core_manager_mut(&mut self) -> &mut Self::CoreManager {
        unreachable!("Platform should never run in NULL.")
    }
}
