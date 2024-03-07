use crate::application::coordinator::Coordinator;
use crate::application::menu::main_menu;
use crate::application::toolbar::Toolbar;
use crate::data::settings::Settings;
use crate::macguiver::application::EventLoopState;
use crate::macguiver::buffer::DrawBuffer;
use crate::platform::{GoLEmPlatform, WindowManager};
use crate::Flags;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use golem_db::Connection;
use sdl3::event::Event;
use sdl3::gamepad::Gamepad;
use sdl3::joystick::Joystick;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

pub mod menu;

pub mod panels;
mod toolbar;
mod widgets;

pub mod coordinator;

use crate::data::paths;

pub struct GoLEmApp {
    toolbar: Toolbar,
    settings: Arc<Settings>,
    database: Arc<Mutex<Connection>>,

    coordinator: Coordinator,

    render_toolbar: bool,

    joysticks: [Option<Joystick>; 32],
    gamepads: [Option<Gamepad>; 32],

    platform: WindowManager,
    main_buffer: DrawBuffer<BinaryColor>,
    toolbar_buffer: DrawBuffer<BinaryColor>,
}

impl GoLEmApp {
    pub fn new(platform: WindowManager) -> Self {
        let settings = Arc::new(Settings::new());

        let database_url = paths::config_root_path().join("golem.sqlite");

        let database = golem_db::establish_connection(&database_url.to_string_lossy())
            .expect("Failed to connect to database");
        let database = Arc::new(Mutex::new(database));
        let toolbar_size = platform.toolbar_dimensions();
        let main_size = platform.main_dimensions();

        // Due to a limitation in Rust language right now, None does not implement Copy
        // when Option<T> does not. This means we can't use it in an array. So we use a
        // constant to work around this.
        let joysticks = {
            const NONE: Option<Joystick> = None;
            [NONE; 32]
        };
        let gamepads = {
            const NONE: Option<Gamepad> = None;
            [NONE; 32]
        };

        Self {
            toolbar: Toolbar::new(settings.clone(), database.clone()),
            coordinator: Coordinator::new(database.clone()),
            render_toolbar: true,
            joysticks,
            gamepads,
            database,
            settings,
            platform,
            main_buffer: DrawBuffer::new(main_size),
            toolbar_buffer: DrawBuffer::new(toolbar_size),
        }
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn run(&mut self, flags: Flags) {
        self.platform.init(&flags);

        // Run the main menu.
        main_menu(self);
    }

    pub fn main_buffer(&mut self) -> &mut DrawBuffer<BinaryColor> {
        &mut self.main_buffer
    }

    pub fn database(&self) -> Arc<Mutex<Connection>> {
        self.database.clone()
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

    pub fn coordinator_mut(&mut self) -> Coordinator {
        self.coordinator.clone()
    }

    pub fn event_loop<R>(
        &mut self,
        mut loop_fn: impl FnMut(&mut Self, &mut EventLoopState) -> Option<R>,
    ) -> R {
        loop {
            self.platform.start_loop();

            let events = self.platform.events();
            for event in events.iter() {
                match event {
                    Event::Quit { .. } => {
                        info!("Quit event received. Quitting...");
                        std::process::exit(0);
                    }
                    Event::JoyDeviceAdded { which, .. } => {
                        let j = self
                            .platform
                            .sdl()
                            .joystick
                            .borrow_mut()
                            .open(*which)
                            .unwrap();
                        if let Some(Some(j)) = self.joysticks.get(*which as usize) {
                            warn!("Joystick {} was already connected. Replacing it.", j.name());
                        }

                        self.joysticks[*which as usize] = Some(j);
                    }
                    Event::JoyDeviceRemoved { which, .. } => {
                        if let Some(None) = self.joysticks.get(*which as usize) {
                            warn!("Joystick #{which} was not detected.");
                        }

                        self.joysticks[*which as usize] = None;
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

            self.platform.update_main(&self.main_buffer);
            if self.render_toolbar && self.toolbar.update() {
                self.toolbar_buffer.clear(BinaryColor::Off).unwrap();
                self.toolbar.draw(&mut self.toolbar_buffer).unwrap();

                if self.settings.invert_toolbar() {
                    self.toolbar_buffer.invert();
                }

                self.platform.update_toolbar(&self.toolbar_buffer);
            }

            self.platform.end_loop();
        }
    }
}
