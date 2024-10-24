use crate::input::commands::CommandId;
use sdl3::event::Event;

pub struct EventLoopState {
    pub(crate) events: Vec<Event>,
    pub(crate) shortcut: Option<CommandId>,
}

impl EventLoopState {
    pub fn events(&self) -> impl Iterator<Item = &Event> + '_ {
        self.events.iter()
    }

    pub fn shortcuts(&self) -> impl Iterator<Item = CommandId> + '_ {
        self.shortcut.iter().copied()
    }
}
