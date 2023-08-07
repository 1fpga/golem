use super::buffer::DrawBuffer;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::primitives::Rectangle;
use embedded_layout::View;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

pub mod clock;
pub mod fps;

pub trait Widget: View {
    type Color: PixelColor;

    fn update(&mut self) -> bool {
        false
    }

    fn draw(&self, target: &mut DrawBuffer<Self::Color>);
}

#[derive(Default)]
pub struct NullWidget<C>(PhantomData<C>);

impl<C: PixelColor> Debug for NullWidget<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NullWidget").finish()
    }
}

impl<C: PixelColor> View for NullWidget<C> {
    fn translate_impl(&mut self, _by: Point) {
        // Do nothing.
    }

    fn bounds(&self) -> Rectangle {
        Rectangle::new(Point::zero(), Size::zero())
    }
}

impl<C: PixelColor> Widget for NullWidget<C> {
    type Color = C;

    fn update(&mut self) -> bool {
        false
    }

    fn draw(&self, _target: &mut DrawBuffer<Self::Color>) {}
}
