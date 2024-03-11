#![allow(unused)]
use embedded_graphics::geometry::AnchorPoint;
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{CornerRadii, PrimitiveStyle, Rectangle, RoundedRectangle};
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::text::{Baseline, Text};
use embedded_layout::align::{horizontal, vertical, Align};

#[derive(Debug, Clone, Copy)]
pub struct Button<'l> {
    label: &'l str,
    font: &'l MonoFont<'l>,
    rectangle: RoundedRectangle,
    focused: bool,
}

impl<'l> Button<'l> {
    pub fn new(label: &'l str, font: &'l MonoFont<'l>) -> Self {
        let label_size = MonoTextStyle::new(font, BinaryColor::On).measure_string(
            label,
            Point::zero(),
            Baseline::Alphabetic,
        );
        let label_size = label_size.bounding_box;

        let button = RoundedRectangle::new(
            label_size.resized(label_size.size + Size::new(10, 4), AnchorPoint::Center),
            CornerRadii::new(Size::new(10, 10)),
        );

        Self {
            label,
            font,
            rectangle: button,
            focused: false,
        }
    }
}

impl<'l> Button<'l> {
    pub fn focus(&mut self) {
        self.focused = true;
    }
    pub fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl<'l> Dimensions for Button<'l> {
    fn bounding_box(&self) -> Rectangle {
        self.rectangle.bounding_box()
    }
}

impl<'l> Transform for Button<'l> {
    fn translate(&self, by: Point) -> Self {
        Self {
            rectangle: self.rectangle.translate(by),
            ..*self
        }
    }

    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.rectangle.translate_mut(by);
        self
    }
}

impl<'l> Drawable for Button<'l> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D: DrawTarget<Color = Self::Color>>(
        &self,
        display: &mut D,
    ) -> Result<Self::Output, D::Error> {
        let on = BinaryColor::On;
        let off = BinaryColor::Off;
        let (bgcolor, color) = if self.focused { (on, off) } else { (off, on) };

        let button_style = if self.focused {
            PrimitiveStyle::with_fill(bgcolor)
        } else {
            PrimitiveStyle::with_stroke(color, 1)
        };

        self.rectangle.into_styled(button_style).draw(display)?;

        Text::new(
            self.label,
            Point::zero(),
            MonoTextStyle::new(self.font, color),
        )
        .align_to(&self.rectangle, horizontal::Center, vertical::Center)
        .draw(display)?;

        Ok(())
    }
}
