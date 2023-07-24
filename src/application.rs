use crate::macgyver::buffer::DrawBuffer;
use crate::window_manager::WindowManager;
use chrono::Timelike;
use embedded_graphics::mono_font::ascii::{FONT_6X9, FONT_8X13};
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use embedded_layout::View;

#[derive(Debug)]
pub enum MiSTerMessage {
    Start,
    Quit,
}

pub struct MiSTer {
    clock: ClockWidget,
}

impl MiSTer {
    pub fn run(&mut self) -> Result<(), String> {
        let mut window_manager = WindowManager::default();
        window_manager.run(self)
    }
}

impl Application for MiSTer {
    type Message = MiSTerMessage;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            clock: ClockWidget::new(),
        }
    }

    fn update(&mut self) {
        self.clock.message(());
    }

    fn draw_title(&self, target: &mut DrawBuffer<BinaryColor>) {
        self.clock.draw(target);
    }

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {}
}

pub trait Widget {
    type Message: std::fmt::Debug + Send;

    fn component_added(&mut self) {}
    fn component_removed(&mut self) {}

    fn size_hint(&self, parent_size: Size) -> Size {
        parent_size
    }

    fn message(&mut self, message: Self::Message) {}

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>);
}

pub trait Application {
    type Message: std::fmt::Debug + Send;

    fn new() -> Self
    where
        Self: Sized;

    fn update(&mut self);

    fn draw_title(&self, target: &mut DrawBuffer<BinaryColor>);
    fn draw(&self, target: &mut DrawBuffer<BinaryColor>);
}

pub struct ClockWidget {
    time: chrono::NaiveTime,
    style: MonoTextStyle<'static, BinaryColor>,
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
    type Message = ();

    fn size_hint(&self, _parent: Size) -> Size {
        self.build_text("00:00:00 AM").size()
    }

    fn message(&mut self, _message: Self::Message) {
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
        let mut text = self.build_text(&time);
        let text_size = text.size();
        let text_width = text_size.width;
        let text_height = text_size.height;
        let text_x = (size.width - text_width);
        let text_y = (size.height - text_height) / 2;
        View::translate_mut(&mut text, Point::new(text_x as i32, text_y as i32));
        text.draw(target).unwrap();
    }
}
