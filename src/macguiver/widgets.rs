use super::buffer::DrawBuffer;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::PixelColor;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::rc::Rc;

pub mod boxed;
pub mod clock;
pub mod group;
pub mod image;
pub mod menu;
pub mod text;

pub trait Widget: Debug {
    type Color: PixelColor;

    fn size_hint(&self, parent_size: Size) -> Size {
        parent_size
    }

    fn update(&mut self) {}

    fn draw(&self, target: &mut DrawBuffer<Self::Color>);
}

impl<T> Widget for Rc<RefCell<T>>
where
    T: Widget + Debug,
{
    type Color = T::Color;

    fn size_hint(&self, parent_size: Size) -> Size {
        self.borrow().size_hint(parent_size)
    }

    fn update(&mut self) {
        self.borrow_mut().update();
    }

    fn draw(&self, target: &mut DrawBuffer<Self::Color>) {
        self.borrow().draw(target);
    }
}

#[derive(Default)]
pub struct NullWidget<C>(PhantomData<C>);

impl<C: PixelColor> Debug for NullWidget<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NullWidget").finish()
    }
}

impl<C: PixelColor> Widget for NullWidget<C> {
    type Color = C;

    fn size_hint(&self, parent_size: Size) -> Size {
        Size::zero()
    }

    fn update(&mut self) {}

    fn draw(&self, _target: &mut DrawBuffer<Self::Color>) {}
}
