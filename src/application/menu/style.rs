use embedded_graphics::primitives::{CornerRadii, RoundedRectangle};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::{DrawTarget, Size},
    primitives::{Primitive, PrimitiveStyle, Rectangle},
    Drawable,
};
use embedded_menu::selection_indicator::{style::IndicatorStyle, Insets};

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
        let mut display_area = display.bounding_box();
        display_area.offset(-5);
        let rect = RoundedRectangle::new(display_area, CornerRadii::new(Size::new(3, 3)));

        rect.into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(display)?;
        Ok(())
    }
}
