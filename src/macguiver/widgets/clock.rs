use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::widgets::text::TextWidget;
use crate::macguiver::widgets::Widget;
use chrono::Timelike;
use embedded_graphics::geometry::Size;
use embedded_graphics::mono_font::ascii::FONT_6X9;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;

#[derive(Debug)]
pub struct ClockWidget {
    time: chrono::NaiveTime,
    inner: TextWidget,
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
            inner: TextWidget::new(
                "00:00:00 AM",
                MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
            ),
        }
    }

    pub fn set_time(&mut self, time: chrono::NaiveTime) {
        self.time = time;

        let hours = time.hour();
        let (hours, am_pm) = if hours > 12 {
            (hours - 12, "PM")
        } else {
            (hours, "AM")
        };

        self.inner.set_text(format!(
            "{:2}:{:02}:{:02} {}",
            hours,
            self.time.minute(),
            self.time.second(),
            am_pm
        ));
    }
}

impl Widget for ClockWidget {
    type Color = BinaryColor;

    fn size_hint(&self, parent: Size) -> Size {
        self.inner.size_hint(parent)
    }

    fn update(&mut self) {
        let time = chrono::Local::now().time();
        if time.second() != self.time.second() {
            self.set_time(time);
        }
    }

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
        self.inner.draw(target);
    }
}
