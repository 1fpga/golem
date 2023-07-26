use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::widgets::Widget;
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::Drawable;
use std::borrow::Cow;
use std::fmt::Debug;

pub mod fps;

#[derive(Debug)]
pub struct TextWidget {
    text: Cow<'static, str>,
    style: MonoTextStyle<'static, BinaryColor>,
}

impl TextWidget {
    pub fn new(
        text: impl Into<Cow<'static, str>>,
        style: MonoTextStyle<'static, BinaryColor>,
    ) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    pub fn set_text(&mut self, text: impl Into<Cow<'static, str>>) {
        self.text = text.into();
    }

    fn build_text(&self) -> Text<'_, MonoTextStyle<'static, BinaryColor>> {
        Text::with_baseline(&self.text, Point::new(0, 0), self.style, Baseline::Top)
    }
}

impl Widget for TextWidget {
    type Color = BinaryColor;

    fn size_hint(&self, _parent: Size) -> Size {
        self.build_text().bounding_box().size
    }

    fn draw(&self, target: &mut DrawBuffer<Self::Color>) {
        self.build_text().draw(target).unwrap();
    }
}
