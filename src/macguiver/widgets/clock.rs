use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::widgets::Widget;
use chrono::Timelike;
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::mono_font::ascii::FONT_6X9;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::Drawable;

#[derive(Debug)]
pub struct ClockWidget {
    time: chrono::NaiveTime,
    style: MonoTextStyle<'static, BinaryColor>,
}

impl Default for ClockWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl ClockWidget {
    pub fn new() -> Self {
        Self {
            time: chrono::Local::now().time(),
            style: MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
        }
    }

    pub fn set_time(&mut self, time: chrono::NaiveTime) {
        self.time = time;
    }

    fn build_text<'a>(&self, text: &'a str) -> Text<'a, MonoTextStyle<'static, BinaryColor>> {
        Text::with_baseline(text, Point::new(0, 0), self.style, Baseline::Top)
    }
}

impl Widget for ClockWidget {
    type Color = BinaryColor;

    fn size_hint(&self, _parent: Size) -> Size {
        self.build_text("00:00:00 AM").bounding_box().size
    }

    fn update(&mut self) {
        self.time = chrono::Local::now().time();
    }

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
        let size = target.size();

        let mut hours = self.time.hour();
        let mut am_pm = "AM";
        if hours > 12 {
            hours -= 12;
            am_pm = "PM";
        }

        let time = format!(
            "{:2}:{:02}:{:02} {}",
            hours,
            self.time.minute(),
            self.time.second(),
            am_pm
        );
        let text = self.build_text(&time);
        text.draw(target).unwrap();
    }
}
