use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::widgets::image::ImageWidget;
use crate::macguiver::widgets::Widget;
use embedded_graphics::geometry::Size;
use embedded_graphics::image::ImageDrawable;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_iconoir::prelude::*;
use embedded_iconoir::Icon;

/// A Widget to display an icon from the Iconoir collection.
#[derive(Debug)]
pub struct IconoirWidget {
    inner: ImageWidget<BinaryColor>,
}

impl IconoirWidget {
    pub fn new<I: IconoirNewIcon<BinaryColor>>() -> Self {
        let sz = I::SIZE;
        let icon: Icon<BinaryColor, I> = Icon::new(BinaryColor::On);
        let mut image = DrawBuffer::new(Size::new(sz, sz));
        icon.draw(&mut image).unwrap();

        Self {
            inner: ImageWidget::new(image),
        }
    }
}

impl Widget for IconoirWidget {
    type Color = BinaryColor;

    fn size_hint(&self, parent_size: Size) -> Size {
        self.inner.size_hint(parent_size)
    }

    fn draw(&self, target: &mut DrawBuffer<Self::Color>) {
        self.inner.draw(target);
    }
}
