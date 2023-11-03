use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::transform::Transform;
use embedded_graphics::Drawable;
use u8g2_fonts::types::{FontColor, HorizontalAlignment, VerticalPosition};
use u8g2_fonts::{Font, FontRenderer};

#[derive(Clone)]
pub struct FontRendererView<'s> {
    pub top_left: Point,
    pub vertical_position: VerticalPosition,
    pub alignment: HorizontalAlignment,
    pub text: &'s str,
    pub renderer: FontRenderer,
}

impl<'s> FontRendererView<'s> {
    pub fn new<F: Font>(
        vertical_position: VerticalPosition,
        alignment: HorizontalAlignment,
        text: &'s str,
    ) -> Self {
        let top_left = Point::zero();
        Self {
            top_left,
            vertical_position,
            alignment,
            text,
            renderer: FontRenderer::new::<F>(),
        }
    }
}

impl<'s> Dimensions for FontRendererView<'s> {
    fn bounding_box(&self) -> Rectangle {
        self.renderer
            .get_rendered_dimensions(self.text, self.top_left, self.vertical_position)
            .unwrap()
            .bounding_box
            .unwrap_or_default()
    }
}

impl<'s> Transform for FontRendererView<'s> {
    fn translate(&self, by: Point) -> Self {
        Self {
            top_left: self.top_left + by,
            renderer: self.renderer.clone(),
            ..*self
        }
    }

    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.top_left += by;
        self
    }
}

impl<'s> Drawable for FontRendererView<'s> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.renderer
            .render(
                self.text,
                self.top_left,
                VerticalPosition::Baseline,
                FontColor::Transparent(BinaryColor::On),
                target,
            )
            .map_err(|e| match e {
                u8g2_fonts::Error::DisplayError(e) => e,
                u8g2_fonts::Error::BackgroundColorNotSupported => unreachable!(),
                u8g2_fonts::Error::GlyphNotFound(c) => unreachable!("{}", c),
            })?;
        Ok(())
    }
}
