use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::widgets::Widget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::primitives::Rectangle;
use std::fmt::{Debug, Formatter};

#[derive(Default)]
pub struct VerticalWidgetGroup<C: PixelColor> {
    inner: Vec<Box<dyn Widget<Color = C>>>,
    margin_left: u32,
    margin_right: u32,
    spacing: u32,
}

impl<C: PixelColor> Debug for VerticalWidgetGroup<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerticalWidgetGroup")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<C: PixelColor> VerticalWidgetGroup<C> {
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            margin_left: 0,
            margin_right: 0,
            spacing: 0,
        }
    }

    pub fn with_margin(mut self, margin: u32) -> Self {
        self.margin_left = margin;
        self.margin_right = margin;
        self
    }

    pub fn with_spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn with(mut self, widget: impl Widget<Color = C> + 'static) -> Self {
        self.inner.push(Box::new(widget));
        self
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn append(&mut self, widget: impl Widget<Color = C> + 'static) {
        self.inner.push(Box::new(widget));
    }

    pub fn remove(&mut self, index: usize) {
        self.inner.remove(index);
    }

    pub fn insert(&mut self, index: usize, widget: impl Widget<Color = C> + 'static) {
        self.inner.insert(index, Box::new(widget));
    }
}

impl<C: PixelColor> Widget for VerticalWidgetGroup<C> {
    type Color = C;

    fn size_hint(&self, parent_size: Size) -> Size {
        // Calculate the total size of the group.
        self.inner.iter().fold(
            Size::new(self.margin_left + self.margin_right, 0),
            |acc, w| {
                let widget_size = w.size_hint(parent_size);
                Size::new(
                    acc.height + widget_size.height + self.spacing,
                    acc.width.max(widget_size.width),
                )
            },
        )
    }

    fn update(&mut self) {
        self.inner.iter_mut().for_each(|w| w.update());
    }

    fn draw(&self, target: &mut DrawBuffer<Self::Color>) {
        let size = target.size();
        let size = Size::new(
            size.width - (self.margin_left + self.margin_right),
            size.height,
        );
        let mut rectangle = Rectangle::new(Point::new(self.margin_left as i32, 0), size);

        for widget in &self.inner {
            let widget_size = widget.size_hint(size);
            rectangle.size = widget_size;

            let mut widget_buffer = target.sub_buffer(rectangle);
            widget.draw(&mut widget_buffer);

            rectangle.top_left.y += widget_size.height as i32 + self.spacing as i32;

            if rectangle.top_left.y > target.size().height as i32 {
                break;
            }
        }
    }
}
