use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::text::Baseline;

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
