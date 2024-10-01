use crate::application::toolbar::Toolbar;
use crate::data::settings::Settings;
use crate::input::commands::CommandId;
use crate::input::shortcut::Shortcut;
use crate::macguiver::application::EventLoopState;
use crate::macguiver::buffer::DrawBuffer;
use crate::platform::de10::De10Platform;
use crate::platform::WindowManager;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::{BinaryColor, Rgb888};
use embedded_graphics::Drawable;
use sdl3::event::Event;
use sdl3::gamepad::Gamepad;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

pub mod menu;

pub mod panels;
mod toolbar;
mod widgets;

pub struct GoLEmApp {
    platform: De10Platform,

    toolbar: Toolbar,
    settings: Arc<Settings>,

    render_toolbar: bool,

    gamepads: [Option<Gamepad>; 32],

    toolbar_buffer: DrawBuffer<BinaryColor>,
    osd_buffer: DrawBuffer<BinaryColor>,

    commands: HashMap<Shortcut, CommandId>,
    command_handler: Option<Box<dyn Fn(CommandId)>>,
}

impl Default for GoLEmApp {
    fn default() -> Self {
        Self::new()
    }
}

impl GoLEmApp {
    pub fn new() -> Self {
        let platform = WindowManager::default();

        let settings = Arc::new(Settings::new());

        let toolbar_size = platform.toolbar_dimensions();
        let osd_size = platform.osd_dimensions();

        // Due to a limitation in Rust language right now, None does not implement Copy
        // when Option<T> does not. This means we can't use it in an array. So we use a
        // constant to work around this.
        let gamepads = {
            const NONE: Option<Gamepad> = None;
            [NONE; 32]
        };

        Self {
            toolbar: Toolbar::new(settings.clone()),
            render_toolbar: true,
            gamepads,
            settings,
            platform,
            toolbar_buffer: DrawBuffer::new(toolbar_size),
            osd_buffer: DrawBuffer::new(osd_size),
            commands: HashMap::new(),
            command_handler: None,
        }
    }

    pub fn set_command_handler(&mut self, handler: impl Fn(CommandId) + 'static) {
        self.command_handler = Some(Box::new(handler));
    }

    pub fn commands(&self) -> &HashMap<Shortcut, CommandId> {
        &self.commands
    }

    pub fn commands_mut(&mut self) -> &mut HashMap<Shortcut, CommandId> {
        &mut self.commands
    }

    pub fn execute_command(&mut self, command_id: CommandId) {
        if let Some(handler) = &self.command_handler {
            handler(command_id);
        }
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn init_platform(&mut self) {
        self.platform.init();
    }

    pub fn main_buffer(&mut self) -> &mut DrawBuffer<Rgb888> {
        self.platform_mut().main_buffer()
    }

    pub fn osd_buffer(&mut self) -> &mut DrawBuffer<BinaryColor> {
        &mut self.osd_buffer
    }

    pub fn platform_mut(&mut self) -> &mut WindowManager {
        &mut self.platform
    }

    pub fn hide_toolbar(&mut self) {
        self.render_toolbar = false;
    }

    pub fn show_toolbar(&mut self) {
        self.render_toolbar = true;
    }

    pub fn add_shortcut(&mut self, shortcut: Shortcut, command: CommandId) {
        self.commands.insert(shortcut, command);
    }

    pub fn remove_shortcut(&mut self, shortcut: Shortcut) {
        self.commands.remove(&shortcut);
    }

    fn draw_inner<R>(&mut self, drawer_fn: impl FnOnce(&mut Self) -> R) -> R {
        self.osd_buffer.clear(BinaryColor::Off).unwrap();
        let result = drawer_fn(self);

        if self.render_toolbar && self.toolbar.update() {
            self.toolbar_buffer.clear(BinaryColor::Off).unwrap();
            self.toolbar.draw(&mut self.toolbar_buffer).unwrap();

            if self.settings.invert_toolbar() {
                self.toolbar_buffer.invert();
            }

            self.platform.update_toolbar(&self.toolbar_buffer);
        }

        // self.platform.update_menu_framebuffer();
        self.platform.update_osd(&self.osd_buffer);

        result
    }

    pub fn draw<R>(&mut self, drawer_fn: impl FnOnce(&mut Self) -> R) -> R {
        self.platform.start_loop();

        let result = self.draw_inner(drawer_fn);

        self.platform.end_loop();
        result
    }

    pub fn draw_loop<R>(
        &mut self,
        mut loop_fn: impl FnMut(&mut Self, &mut EventLoopState) -> Option<R>,
    ) -> R {
        self.event_loop(|s, state| s.draw(|s| loop_fn(s, state)))
    }

    pub fn event_loop<R>(
        &mut self,
        mut loop_fn: impl FnMut(&mut Self, &mut EventLoopState) -> Option<R>,
    ) -> R {
        loop {
            let events = self.platform.events();
            for event in events.iter() {
                match event {
                    Event::Quit { .. } => {
                        info!("Quit event received. Quitting...");
                        std::process::exit(0);
                    }
                    Event::ControllerDeviceAdded { which, .. } => {
                        let g = self
                            .platform
                            .sdl()
                            .gamepad
                            .borrow_mut()
                            .open(*which)
                            .unwrap();
                        if let Some(Some(g)) = self.gamepads.get(*which as usize) {
                            warn!("Gamepad {} was already connected. Replacing it.", g.name());
                        }

                        self.gamepads[*which as usize] = Some(g);
                    }
                    Event::ControllerDeviceRemoved { which, .. } => {
                        if let Some(None) = self.gamepads.get(*which as usize) {
                            warn!("Gamepad #{which} was not detected.");
                        }

                        self.gamepads[*which as usize] = None;
                    }
                    _ => {}
                }
            }

            let mut state = EventLoopState::new(events);

            if let Some(r) = loop_fn(self, &mut state) {
                break r;
            }
        }
    }
}
