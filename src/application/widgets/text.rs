use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::text::Baseline;
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

pub fn wrap_text(text: &str, width: u32, font: &MonoFont) -> Vec<String> {
    let style = MonoTextStyle::new(font, BinaryColor::On);

    let mut lines = Vec::new();
    let mut line = String::new();
    let mut line_width = 0;

    let space_w = style
        .measure_string(" ", Point::zero(), Baseline::Bottom)
        .bounding_box
        .size
        .width;

    for word in text.split_whitespace() {
        let word_width = style
            .measure_string(word, Point::zero(), Baseline::Bottom)
            .bounding_box
            .size
            .width;

        if word_width > width {
            lines.push(line);
            line = String::new();
            line_width = 0;

            for i in (0..word_width as usize).step_by(width as usize) {
                let end = (i + width as usize).min(word_width as usize);
                lines.push((&word[i..end]).to_string());
            }

            continue;
        } else if line_width + word_width > width {
            lines.push(line);
            line = String::new();
            line_width = 0;
        }

        line.push_str(word);
        line_width += word_width;
        line.push(' ');
        line_width += space_w;
    }

    lines.push(line);

    lines
}
