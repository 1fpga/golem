use crate::macguiver::buffer::DrawBuffer;
use crate::platform::MiSTerPlatform;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::BinaryColor;
use sdl3::event::Event;

pub struct NullCoreManager;

impl super::CoreManager for NullCoreManager {
    fn load_program(&mut self, _program: &[u8]) -> Result<(), String> {
        unreachable!("Platform should never run in NULL.")
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

    fn start_loop(&mut self) {}
    fn end_loop(&mut self) {}

    fn core_manager_mut(&mut self) -> &mut Self::CoreManager {
        unreachable!("Platform should never run in NULL.")
    }
}
