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
