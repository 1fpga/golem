use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::platform::sdl::SdlPlatform;
use crate::platform::MiSTerPlatform;
use crate::types::StatusBitMap;
use crate::utils::config_string::ConfigMenu;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::BinaryColor;
use sdl3::event::Event;
use std::path::Path;

pub struct NullCore;

impl super::Core for NullCore {
    fn name(&self) -> &str {
        ""
    }

    fn menu_options(&self) -> &[ConfigMenu] {
        unreachable!()
    }

    fn status_bits(&self) -> StatusBitMap {
        unreachable!()
    }

    fn set_status_bits(&mut self, bits: StatusBitMap) {
        unreachable!()
    }

    fn send_key(&mut self, key: u8) {
        unreachable!()
    }

    fn sdl_joy_button_down(&mut self, joystick_idx: u8, button: u8) {
        unreachable!()
    }

    fn sdl_joy_button_up(&mut self, joystick_idx: u8, button: u8) {
        unreachable!()
    }
}

pub struct NullCoreManager;

impl super::CoreManager for NullCoreManager {
    type Core = NullCore;

    fn load_program(&mut self, path: &Path) -> Result<Self::Core, String> {
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

impl MiSTerPlatform for NullPlatform {
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
