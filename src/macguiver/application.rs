use super::buffer::DrawBuffer;
use crate::platform::PlatformState;
use embedded_graphics::pixelcolor::PixelColor;

pub enum UpdateResult {
    Redraw(bool, bool),
    NoRedraw,
    Quit,
}

pub trait Application {
    type Color: PixelColor;

    fn new() -> Self
    where
        Self: Sized;

    fn update(&mut self, state: &PlatformState) -> UpdateResult;

    fn draw_title(&self, target: &mut DrawBuffer<Self::Color>);
    fn draw_main(&self, target: &mut DrawBuffer<Self::Color>);
}
