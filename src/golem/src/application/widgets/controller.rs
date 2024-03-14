use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point};
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::pixelcolor::{BinaryColor, PixelColor};
use embedded_graphics::primitives::{Circle, Primitive, PrimitiveStyle, Rectangle, Styled};
use embedded_graphics::text::Text;
use embedded_graphics::transform::Transform;
use embedded_graphics::Drawable;
use embedded_layout::View;

#[derive(Debug, Copy, Clone)]
pub struct ControllerButton<'l, C>
where
    C: PixelColor,
{
    label: &'l str,
    font: &'l MonoFont<'l>,
    label_color: C,
    circle: Styled<Circle, PrimitiveStyle<C>>,
}

impl<'l> ControllerButton<'l, BinaryColor> {
    pub fn new(label: &'l str, font: &'l MonoFont<'l>) -> Self {
        let top_left = Point::zero();
        let circle = Circle::new(top_left, font.character_size.width + 4);
        let circle = circle.into_styled(PrimitiveStyle::with_fill(BinaryColor::On));

        Self {
            label,
            font,
            label_color: BinaryColor::Off,
            circle,
        }
    }
}

impl<'l, C> Dimensions for ControllerButton<'l, C>
where
    C: PixelColor,
{
    fn bounding_box(&self) -> Rectangle {
        self.circle.bounding_box()
    }
}

impl<'l, C> Transform for ControllerButton<'l, C>
where
    C: PixelColor,
{
    fn translate(&self, by: Point) -> Self {
        Self {
            circle: self.circle.translate(by),
            ..*self
        }
    }

    fn translate_mut(&mut self, by: Point) -> &mut Self {
        View::translate_mut(&mut self.circle, by);
        self
    }
}

impl<'l, C> Drawable for ControllerButton<'l, C>
where
    C: PixelColor,
{
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.circle.translate(Point::new(0, 1)).draw(target)?;

        let bottom_left = self.circle.primitive.top_left
            + Point::new(3, self.circle.primitive.diameter as i32 - 2);

        Text::new(
            self.label,
            bottom_left,
            MonoTextStyle::new(self.font, self.label_color),
        )
        .draw(target)?;
        Ok(())
    }
}
