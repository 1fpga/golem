use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::RgbColor;
use embedded_graphics::pixelcolor::{BinaryColor, Rgb888};
use embedded_graphics::prelude::Size;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use embedded_layout::View;

use embedded_menu::items::MenuListItem;

#[derive(Clone)]
pub struct SectionSeparator {
    line: Rectangle,
}

impl Default for SectionSeparator {
    fn default() -> Self {
        Self::new()
    }
}

impl embedded_menu::items::Marker for SectionSeparator {}

impl<R> MenuListItem<R> for SectionSeparator {
    fn value_of(&self) -> R {
        unreachable!()
    }

    fn interact(&mut self) -> R {
        unreachable!()
    }

    fn set_style(&mut self, _text_style: &MonoTextStyle<'_, BinaryColor>) {
        // Nothing to do.
    }

    fn selectable(&self) -> bool {
        false
    }

    fn draw_styled<DIS>(
        &self,
        _style: &MonoTextStyle<'_, BinaryColor>,
        display: &mut DIS,
    ) -> Result<(), DIS::Error>
    where
        DIS: DrawTarget<Color = BinaryColor>,
    {
        self.line.translate(Point::new(0, 1)).draw_styled(
            &PrimitiveStyle::with_stroke(Rgb888::WHITE.into(), 1),
            display,
        )
    }
}

impl SectionSeparator {
    pub fn new() -> Self {
        Self {
            line: Rectangle::new(Point::zero(), Size::new(300, 1)),
        }
    }
}

impl View for SectionSeparator {
    fn translate_impl(&mut self, by: Point) {
        View::translate_mut(&mut self.line, by);
    }

    fn bounds(&self) -> Rectangle {
        let mut bounds = self.line.bounds();
        bounds.size.height += 2;
        bounds
    }
}
