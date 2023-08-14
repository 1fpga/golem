use crate::macguiver::application::Application;
use crate::platform::MiSTerPlatform;
use embedded_graphics::pixelcolor::BinaryColor;

#[derive(Default)]
pub struct NullWindowManager {}

impl MiSTerPlatform for NullWindowManager {
    type Color = BinaryColor;

    fn run(&mut self, _app: &mut impl Application<Color = Self::Color>) {
        unreachable!("Platform should never run in NULL.")
    }
}

pub use NullWindowManager as PlatformWindowManager;
