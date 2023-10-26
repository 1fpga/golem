use crate::application::menu::main_menu;
use crate::application::toolbar::Toolbar;
use crate::data::settings::Settings;
use crate::macguiver::application::{Application, EventLoopState};
use crate::macguiver::buffer::DrawBuffer;
use crate::main_inner::Flags;
use crate::platform::{MiSTerPlatform, WindowManager};
pub use cores::CoreManager;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use mister_db::Connection;
use sdl3::event::Event;
use sdl3::joystick::Joystick;
use std::sync::{Arc, Mutex, RwLock};
use tracing::{info, warn};

// mod icons;
pub mod menu;

mod panels;
mod toolbar;
mod widgets;

mod cores;

use crate::data::paths;

pub struct MiSTer {
    toolbar: Toolbar,
    settings: Arc<Settings>,
    database: Arc<Mutex<Connection>>,

    render_toolbar: bool,

    joysticks: [Option<Joystick>; 16],

    core_manager: Arc<RwLock<CoreManager>>,

    pub platform: WindowManager,
    main_buffer: DrawBuffer<BinaryColor>,
    toolbar_buffer: DrawBuffer<BinaryColor>,
}

impl MiSTer {
    pub fn new(platform: WindowManager) -> Self {
        let settings = Arc::new(Settings::new());

        let database_url = paths::config_root_path().join("golem.sqlite");

        let database = mister_db::establish_connection(&database_url.to_string_lossy())
            .expect("Failed to connect to database");
        let database = Arc::new(Mutex::new(database));
        let toolbar_size = platform.toolbar_dimensions();
        let main_size = platform.main_dimensions();
        let core_manager = CoreManager::new(database.clone());

        // Due to a limitation in Rust language right now, None does not implement Copy
        // when Option<T> does not. This means we can't use it in an array. So we use a
        // constant to work around this.
        const NONE: Option<Joystick> = None;
        let joysticks = [NONE; 16];

        Self {
            toolbar: Toolbar::new(settings.clone(), database.clone()),
            render_toolbar: true,
            core_manager: Arc::new(RwLock::new(core_manager)),
            joysticks,
            database,
            settings,
            platform,
            main_buffer: DrawBuffer::new(main_size),
            toolbar_buffer: DrawBuffer::new(toolbar_size),
        }
    }
}

impl Application for MiSTer {
    type Color = BinaryColor;
    type Platform = WindowManager;

    fn settings(&self) -> &Settings {
        &self.settings
    }

    fn run(&mut self, flags: Flags) {
        self.platform.init(&flags);

        // Run the main menu.
        main_menu(self);
    }

    fn main_buffer(&mut self) -> &mut DrawBuffer<Self::Color> {
        &mut self.main_buffer
    }

    fn database(&self) -> Arc<Mutex<Connection>> {
        self.database.clone()
    }

    fn core_manager(&self) -> Arc<RwLock<CoreManager>> {
        Arc::clone(&self.core_manager)
    }

    fn platform(&self) -> &Self::Platform {
        &self.platform
    }

    fn platform_mut(&mut self) -> &mut Self::Platform {
        &mut self.platform
    }

    fn hide_toolbar(&mut self) {
        self.render_toolbar = false;
    }

    fn show_toolbar(&mut self) {
        self.render_toolbar = true;
    }

    fn event_loop<R>(
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
