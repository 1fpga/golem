use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, OriginDimensions, Point, Size};
use embedded_graphics::mono_font::ascii::FONT_6X9;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::transform::Transform;
use embedded_graphics::Drawable;

#[derive(Debug, Clone)]
pub struct DateTimeWidget {
    bounds: Rectangle,
    date_time: chrono::DateTime<chrono::Local>,
    time_format: String,
    style: MonoTextStyle<'static, BinaryColor>,
    text: String,
}

impl Default for DateTimeWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl DateTimeWidget {
    pub fn new() -> Self {
        let time_format = "%a %d %b %H:%M:%S".to_string();
        let date_time = chrono::Local::now();
        let text = date_time.format(&time_format).to_string();
        let style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);

        Self {
            bounds: Text::with_baseline(&text, Point::zero(), style, Baseline::Top).bounding_box(),
            date_time,
            time_format,
            style,
            text,
        }
    }

    pub fn set_time(&mut self, date_time: chrono::DateTime<chrono::Local>) {
        self.date_time = date_time;
        self.text = self.date_time.format(&self.time_format).to_string();
    }

    pub fn update(&mut self) {
        let date_time = chrono::Local::now();
        if date_time != self.date_time {
            self.set_time(date_time);
        }
    }
}

impl Dimensions for DateTimeWidget {
    fn bounding_box(&self) -> Rectangle {
        self.bounds
    }
}

impl Transform for DateTimeWidget {
    fn translate(&self, by: Point) -> Self {
        let mut new = self.clone();
        new.translate_mut(by);
        new
    }

    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.bounds.top_left += by;
        self
    }
}

impl Drawable for DateTimeWidget {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        Text::with_baseline(&self.text, self.bounds.top_left, self.style, Baseline::Top)
            .draw(target)?;
        Ok(())
    }
}
