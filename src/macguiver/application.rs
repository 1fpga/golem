use super::buffer::DrawBuffer;
use crate::platform::PlatformState;
use embedded_graphics::pixelcolor::PixelColor;

pub trait Application {
    type Color: PixelColor;

    fn new() -> Self
    where
        Self: Sized;

    fn update(&mut self, state: &PlatformState);

    fn draw_title(&self, target: &mut DrawBuffer<Self::Color>);
    fn draw(&self, target: &mut DrawBuffer<Self::Color>);
}
