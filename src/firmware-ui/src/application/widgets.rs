//! MiSTer specific views.
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::{BinaryColor, PixelColor};
use embedded_graphics::prelude::Dimensions;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::transform::Transform;
use embedded_graphics::Drawable;

pub mod button;
pub mod controller;
pub mod menu;
pub mod network;
pub mod opt;
pub mod text;

#[derive(Debug, Copy, Clone)]
pub struct EmptyView<C = BinaryColor>(Point, core::marker::PhantomData<C>);

impl<C> Default for EmptyView<C> {
    fn default() -> Self {
        Self(Point::zero(), core::marker::PhantomData)
    }
}

impl<C> Dimensions for EmptyView<C> {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(self.0, Size::zero())
    }
}

impl<C> Transform for EmptyView<C> {
    fn translate(&self, by: Point) -> Self {
        Self(self.0 + by, core::marker::PhantomData)
    }

    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.0 += by;
        self
    }
}

impl<C: PixelColor> Drawable for EmptyView<C> {
    type Color = C;
    type Output = ();

    fn draw<D>(&self, _target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        Ok(())
    }
}
