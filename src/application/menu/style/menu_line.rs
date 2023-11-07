use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::pixelcolor::RgbColor;
use embedded_graphics::prelude::Size;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use embedded_layout::View;

use embedded_menu::{
    interaction::InputAdapterSource,
    selection_indicator::{style::IndicatorStyle, SelectionIndicatorController},
    Marker, MenuItem, MenuStyle,
};

#[derive(Clone)]
pub struct SectionSeparator {
    line: Rectangle,
}

impl Default for SectionSeparator {
    fn default() -> Self {
        Self::new()
    }
}

impl Marker for SectionSeparator {}

impl<R> MenuItem<R> for SectionSeparator {
    fn value_of(&self) -> R {
        unreachable!()
    }

    fn interact(&mut self) -> R {
        unreachable!()
    }

    fn set_style<C, S, IT, P>(&mut self, _style: &MenuStyle<C, S, IT, P, R>)
    where
        C: PixelColor,
        S: IndicatorStyle,
        IT: InputAdapterSource<R>,
        P: SelectionIndicatorController,
    {
        // Nothing to do.
    }

    fn title(&self) -> &str {
        ""
    }

    fn details(&self) -> &str {
        ""
    }

    fn value(&self) -> &str {
        ""
    }

    fn selectable(&self) -> bool {
        false
    }

    fn draw_styled<C, S, IT, P, DIS>(
        &self,
        _style: &MenuStyle<C, S, IT, P, R>,
        display: &mut DIS,
    ) -> Result<(), DIS::Error>
    where
        C: PixelColor + From<Rgb888>,
        S: IndicatorStyle,
        IT: InputAdapterSource<R>,
        P: SelectionIndicatorController,
        DIS: DrawTarget<Color = C>,
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
