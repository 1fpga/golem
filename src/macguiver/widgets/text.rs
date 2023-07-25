use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::widgets::Widget;
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::Drawable;
use std::fmt::Debug;

#[derive(Debug)]
pub struct TextWidget<C: PixelColor> {
    text: String,
    style: MonoTextStyle<'static, C>,
}

impl<C: PixelColor> TextWidget<C> {
    pub fn new(text: String, style: MonoTextStyle<'static, C>) -> Self {
        Self { text, style }
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    fn build_text(&self) -> Text<'_, MonoTextStyle<'static, C>> {
        Text::with_baseline(&self.text, Point::new(0, 0), self.style, Baseline::Top)
    }
}

impl<C: PixelColor + Debug> Widget for TextWidget<C> {
    type Color = C;

    fn size_hint(&self, _parent: Size) -> Size {
        self.build_text().bounding_box().size
    }

    fn draw(&self, target: &mut DrawBuffer<Self::Color>) {
        self.build_text().draw(target).unwrap();
    }
}
