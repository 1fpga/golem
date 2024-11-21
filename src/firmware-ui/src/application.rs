use crate::application::toolbar::Toolbar;
use crate::data::settings::UiSettings;
use crate::input::commands::CommandId;
use crate::input::shortcut::Shortcut;
use crate::input::InputState;
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
use tracing::{debug, info, trace, warn};

pub mod menu;

pub mod panels;
mod toolbar;
mod widgets;

pub struct OneFpgaApp {
    platform: De10Platform,

    toolbar: Toolbar,

    render_toolbar: bool,

    gamepads: [Option<Gamepad>; 32],

    toolbar_buffer: DrawBuffer<BinaryColor>,
    osd_buffer: DrawBuffer<BinaryColor>,

    input_state: InputState,
    shortcuts: HashMap<Shortcut, CommandId>,

    ui_settings: UiSettings,
}

impl Default for OneFpgaApp {
    fn default() -> Self {
        Self::new()
    }
}

impl OneFpgaApp {
    pub fn new() -> Self {
        let platform = WindowManager::default();

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
            toolbar: Toolbar::new(),
            render_toolbar: true,
            gamepads,
            platform,
            toolbar_buffer: DrawBuffer::new(toolbar_size),
            osd_buffer: DrawBuffer::new(osd_size),
            input_state: InputState::default(),
            shortcuts: Default::default(),
            ui_settings: UiSettings::default(),
        }
    }

    pub fn add_shortcut(&mut self, shortcut: Shortcut, command: CommandId) {
        self.shortcuts.insert(shortcut, command);
    }

    pub fn remove_shortcut(&mut self, shortcut: &Shortcut) -> Option<CommandId> {
        self.shortcuts.remove(shortcut)
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

    pub fn ui_settings(&self) -> &UiSettings {
        &self.ui_settings
    }

    pub fn ui_settings_mut(&mut self) -> &mut UiSettings {
        &mut self.ui_settings
    }

    fn draw_inner<R>(&mut self, drawer_fn: impl FnOnce(&mut Self) -> R) -> R {
        self.osd_buffer.clear(BinaryColor::Off).unwrap();
        let result = drawer_fn(self);

        if self.render_toolbar && self.toolbar.update(*self.ui_settings()) {
            self.toolbar_buffer.clear(BinaryColor::Off).unwrap();
            self.toolbar.draw(&mut self.toolbar_buffer).unwrap();

            if self.ui_settings.invert_toolbar() {
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
        mut loop_fn: impl FnMut(&mut Self, EventLoopState) -> Option<R>,
    ) -> R {
        self.event_loop(|s, state| s.draw(|s| loop_fn(s, state)))
    }

    pub fn event_loop<R>(
        &mut self,
        mut loop_fn: impl FnMut(&mut Self, EventLoopState) -> Option<R>,
    ) -> R {
        let mut triggered_commands = vec![];

        loop {
            let events = self.platform.events();

            let mut longest_shortcut = Shortcut::default();
            let mut shortcut = None;

            let mut check_shortcuts = false;
            for event in events.iter() {
                trace!(?event, "Event received");

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
                        debug!(name = g.name(), mapping = g.mapping(), "Gamepad connected");

                        self.gamepads[*which as usize] = Some(g);
                    }
                    Event::ControllerDeviceRemoved { which, .. } => {
                        if let Some(None) = self.gamepads.get(*which as usize) {
                            warn!("Gamepad #{which} was not detected.");
                        }

                        self.gamepads[*which as usize] = None;
                    }
                    Event::KeyDown {
                        scancode: Some(scancode),
                        repeat,
                        ..
                    } => {
                        if !repeat {
                            self.input_state.key_down(*scancode);
                            check_shortcuts = true;
                        }
                    }
                    Event::KeyUp {
                        scancode: Some(scancode),
                        ..
                    } => {
                        self.input_state.key_up(*scancode);
                        check_shortcuts = true;
                    }
                    Event::ControllerButtonDown { which, button, .. } => {
                        self.input_state.controller_button_down(*which, *button);
                        check_shortcuts = true;
                    }
                    Event::ControllerButtonUp { which, button, .. } => {
                        self.input_state.controller_button_up(*which, *button);
                        check_shortcuts = true;
                    }
                    Event::ControllerAxisMotion {
                        which, axis, value, ..
                    } => {
                        self.input_state
                            .controller_axis_motion(*which, *axis, *value);
                        check_shortcuts = true;
                    }
                    _ => {}
                }
            }
            if check_shortcuts {
                for (s, id) in &self.shortcuts {
                    if s.matches(&self.input_state) {
                        if triggered_commands.contains(id) {
                            continue;
                        }

                        debug!(id = ?*id, shortcut = ?s, input_state = ?self.input_state, "Command triggered");
                        triggered_commands.push(*id);

                        if s > &longest_shortcut {
                            longest_shortcut = s.clone();
                            shortcut = Some(*id);
                        }
                    } else {
                        triggered_commands.retain(|x| x != id);
                    }
                }
            }

            if let Some(r) = loop_fn(self, EventLoopState { events, shortcut }) {
                break r;
            }
        }
    }
}
