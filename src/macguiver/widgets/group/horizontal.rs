use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::widgets::Widget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::primitives::Rectangle;
use std::fmt::{Debug, Formatter};

#[derive(Default)]
pub struct HorizontalWidgetGroup<C: PixelColor> {
    inner: Vec<Box<dyn Widget<Color = C>>>,
    margin_top: u32,
    margin_bottom: u32,
    spacing: u32,
}

impl<C: PixelColor> Debug for HorizontalWidgetGroup<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HorizontalWidgetGroup")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<C: PixelColor> HorizontalWidgetGroup<C> {
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            margin_top: 0,
            margin_bottom: 0,
            spacing: 0,
        }
    }

    pub fn with_margin(mut self, margin: u32) -> Self {
        self.margin_top = margin;
        self.margin_bottom = margin;
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

impl<C: PixelColor> Widget for HorizontalWidgetGroup<C> {
    type Color = C;

    fn size_hint(&self, parent_size: Size) -> Size {
        // Calculate the total size of the group.
        self.inner.iter().fold(
            Size::new(0, self.margin_top + self.margin_bottom),
            |acc, w| {
                let widget_size = w.size_hint(parent_size);
                Size::new(
                    acc.width + widget_size.width + self.spacing,
                    acc.height.max(widget_size.height),
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
            size.width,
            size.height - (self.margin_top + self.margin_bottom),
        );
        let mut rectangle = Rectangle::new(Point::new(0, self.margin_top as i32), size);

        for widget in &self.inner {
            let widget_size = widget.size_hint(size);
            rectangle.size = widget_size;

            let mut widget_buffer = target.sub_buffer(rectangle);
            widget.draw(&mut widget_buffer);

            rectangle.top_left.x += widget_size.width as i32 + self.spacing as i32;

            if rectangle.top_left.x > target.size().width as i32 {
                break;
            }
        }
    }
}
