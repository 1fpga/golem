use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::widgets::Widget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::primitives::Rectangle;
use std::fmt::Debug;

#[derive(Debug, Default)]
pub enum VerticalAlignment {
    #[default]
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Default)]
pub enum HorizontalAlignment {
    #[default]
    Left,
    Center,
    Right,
}

/// Acts as a Box surrounding a Widget. Will forward all calls to the inner Widget.
/// This can be used when the type of Widget is not known at compile time.
///
/// It also includes alignment and margins.
#[derive(Debug)]
pub struct BoxedWidget<C: PixelColor> {
    inner: Box<dyn Widget<Color = C>>,
    margin_top: i32,
    margin_right: i32,
    margin_bottom: i32,
    margin_left: i32,
    vertical: VerticalAlignment,
    horizontal: HorizontalAlignment,
}

impl<C: PixelColor> BoxedWidget<C> {
    pub fn new(inner: impl Widget<Color = C> + 'static) -> Self {
        Self {
            inner: Box::new(inner),
            margin_top: 0,
            margin_left: 0,
            margin_bottom: 0,
            margin_right: 0,
            vertical: Default::default(),
            horizontal: Default::default(),
        }
    }

    pub fn aligned(mut self, vertical: VerticalAlignment, horizontal: HorizontalAlignment) -> Self {
        self.vertical = vertical;
        self.horizontal = horizontal;
        self
    }

    pub fn with_margin(mut self, margin: i32) -> Self {
        self.margin_top = margin;
        self.margin_left = margin;
        self.margin_bottom = margin;
        self.margin_right = margin;
        self
    }

    pub fn with_margin_tuple(mut self, margin: (i32, i32, i32, i32)) -> Self {
        self.margin_top = margin.0;
        self.margin_right = margin.1;
        self.margin_bottom = margin.2;
        self.margin_left = margin.3;
        self
    }

    fn inner_size(&self, size: Size) -> Size {
        Size::new(
            ((size.width as i32) - (self.margin_left + self.margin_right)).max(0) as u32,
            ((size.height as i32) - (self.margin_top + self.margin_bottom)).max(0) as u32,
        )
    }
}

impl<C: PixelColor + Debug> Widget for BoxedWidget<C> {
    type Color = C;

    fn size_hint(&self, parent_size: Size) -> Size {
        // Update parent_size to remove margins.
        let size = self.inner_size(parent_size);
        self.inner.size_hint(size)
    }

    fn update(&mut self) {
        self.inner.update();
    }

    fn draw(&self, target: &mut DrawBuffer<Self::Color>) {
        let size = target.size();
        let size = self.inner_size(size);
        let inner_size = self.inner.size_hint(size);

        let y = self.margin_top
            + match self.vertical {
                VerticalAlignment::Top => 0,
                VerticalAlignment::Bottom => (size.height as i32) - (inner_size.height as i32),
                VerticalAlignment::Middle => {
                    ((size.height as i32) - (inner_size.height as i32)) / 2
                }
            };

        let x = self.margin_left
            + match self.horizontal {
                HorizontalAlignment::Left => 0,
                HorizontalAlignment::Right => (size.width as i32) - (inner_size.width as i32),
                HorizontalAlignment::Center => {
                    ((size.width as i32) - (inner_size.width as i32)) / 2
                }
            };
        let mut inner_target = target.sub_buffer(Rectangle::new(Point::new(x, y), inner_size));

        self.inner.draw(&mut inner_target);
    }
}
