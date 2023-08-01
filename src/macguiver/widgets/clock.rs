use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::widgets::text::TextWidget;
use crate::macguiver::widgets::Widget;
use embedded_graphics::geometry::Size;
use embedded_graphics::mono_font::ascii::FONT_6X9;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;

#[derive(Debug)]
pub struct DateTimeWidget {
    date_time: chrono::DateTime<chrono::Local>,
    time_format: String,
    inner: TextWidget,
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
        let date_time_str = date_time.format(&time_format).to_string();

        Self {
            date_time,
            time_format,
            inner: TextWidget::new(
                date_time_str,
                MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
            ),
        }
    }

    pub fn set_time(&mut self, date_time: chrono::DateTime<chrono::Local>) {
        self.date_time = date_time;
        self.inner
            .set_text(date_time.format(&self.time_format).to_string());
    }
}

impl Widget for DateTimeWidget {
    type Color = BinaryColor;

    fn size_hint(&self, parent: Size) -> Size {
        self.inner.size_hint(parent)
    }

    fn update(&mut self) {
        let date_time = chrono::Local::now();
        if date_time != self.date_time {
            self.set_time(date_time);
        }
    }

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
        self.inner.draw(target);
    }
}
