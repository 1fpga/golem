use embedded_graphics::mono_font::ascii::{FONT_6X10, FONT_8X13};
use embedded_graphics::primitives::{CornerRadii, RoundedRectangle};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::{DrawTarget, Size},
    primitives::{Primitive, PrimitiveStyle, Rectangle},
    Drawable,
};
use embedded_menu::interaction::programmed::Programmed;
use embedded_menu::selection_indicator::{style::IndicatorStyle, AnimatedPosition, Insets};
use embedded_menu::{DisplayScrollbar, MenuStyle};

pub fn menu_style() -> MenuStyle<BinaryColor, RoundedRect, Programmed, AnimatedPosition> {
    MenuStyle::new(BinaryColor::On)
        .with_animated_selection_indicator(5)
        .with_selection_indicator(RoundedRect)
        .with_scrollbar_style(DisplayScrollbar::Auto)
        .with_title_font(&FONT_8X13)
        .with_font(&FONT_6X10)
}

#[derive(Clone, Copy)]
pub struct RoundedRect;

impl IndicatorStyle for RoundedRect {
    type Shape = Rectangle;
    type State = ();

    fn margin(&self, _state: &Self::State, _height: u32) -> Insets {
        Insets::new(4, 3, 4, 3)
    }

    fn shape(&self, _state: &Self::State, bounds: Rectangle, _fill_width: u32) -> Self::Shape {
        bounds
    }

    fn draw<D>(
        &self,
        _state: &Self::State,
        _fill_width: u32,
        display: &mut D,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let display_area = display.bounding_box().offset(-2);
        let rect = RoundedRectangle::new(display_area, CornerRadii::new(Size::new(3, 3)));

        rect.into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(display)?;
        Ok(())
    }
}
