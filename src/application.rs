use crate::macgyver::buffer::DrawBuffer;
use crate::window_manager::WindowManager;
use chrono::Timelike;
use embedded_graphics::mono_font::ascii::FONT_6X9;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;
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

    fn update(&mut self) {}

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
        self.clock.draw(target);
    }
}

pub trait Widget<Message> {
    fn component_added(&mut self) {}
    fn component_removed(&mut self) {}

    fn size_hint(&self, parent_size: Size) -> Size {
        parent_size
    }

    fn message(&mut self, message: Message) -> Option<Command<Message>> {
        None
    }

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>);
}

#[derive(Debug)]
pub struct Command<Message> {
    pub message: Option<Message>,
}

impl<Message> Command<Message> {
    pub fn none() -> Self {
        Self { message: None }
    }

    pub fn message(message: Message) -> Self {
        Self {
            message: Some(message),
        }
    }
}

pub trait Application {
    type Message: std::fmt::Debug + Send;

    fn new() -> Self
    where
        Self: Sized;

    fn update(&mut self);

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>);
}

pub struct ClockWidget {
    time: chrono::NaiveTime,
}

impl ClockWidget {
    pub fn new() -> Self {
        Self {
            time: chrono::Local::now().time(),
        }
    }

    pub fn set_time(&mut self, time: chrono::NaiveTime) {
        self.time = time;
    }
}

impl Widget<()> for ClockWidget {
    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
        let size = target.size();

        let mut hours = self.time.hour();
        let mut am_pm = "AM";
        if hours > 12 {
            hours -= 12;
            am_pm = "PM";
        }

        let mut time = format!(
            "{:02}:{:02}:{:02} {}",
            hours,
            self.time.minute(),
            self.time.second(),
            am_pm
        );
        let text_style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);
        let mut text = Text::new(&time, Point::new(0, 0), text_style);
        let text_size = text.size();
        let text_width = text_size.width;
        let text_height = text_size.height;
        let text_x = (size.width - text_width) / 2;
        let text_y = (size.height - text_height) / 2;
        View::translate_mut(&mut text, Point::new(text_x as i32, text_y as i32));
        // text.translate_mut(Point::new(text_x as i32, text_y as i32));
        text.draw(target).unwrap();
    }
}
