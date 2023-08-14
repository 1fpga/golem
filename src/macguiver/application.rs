use crate::application::TopLevelViewType;
use crate::data::settings::Settings;
use crate::macguiver::buffer::DrawBuffer;
use crate::main_inner::Flags;
use crate::platform::{MiSTerPlatform, PlatformState};
use embedded_graphics::pixelcolor::PixelColor;
use sdl3::event::Event;

pub struct EventLoopState {
    events: Vec<Event>,
}

impl EventLoopState {
    pub fn new(events: Vec<Event>) -> Self {
        Self { events }
    }

    pub fn events(&mut self) -> impl Iterator<Item = Event> + '_ {
        self.events.iter().cloned()
    }
}

pub trait Application {
    type Color: PixelColor;

    fn settings(&self) -> &Settings;
    fn run(&mut self, flags: Flags);

    fn main_buffer(&mut self) -> &mut DrawBuffer<Self::Color>;

    fn event_loop(
        &mut self,
        loop_fn: impl FnMut(&mut Self, &mut EventLoopState) -> Option<TopLevelViewType>,
    ) -> TopLevelViewType;
}
