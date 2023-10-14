use crate::application::menu::style::MenuReturn;
use crate::application::menu::{cores_menu_panel, main_menu};
use crate::application::panels::input_tester::input_tester;
use crate::application::panels::settings::settings_panel;
use crate::application::toolbar::Toolbar;
use crate::data::settings::Settings;
use crate::macguiver::application::{Application, EventLoopState};
use crate::macguiver::buffer::DrawBuffer;
use crate::main_inner::Flags;
use crate::platform::{MiSTerPlatform, WindowManager};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use mister_db::Connection;
use sdl3::event::Event;
use sdl3::joystick::Joystick;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

// mod icons;
pub mod menu;

mod panels;
mod toolbar;
mod widgets;

mod cores;

use crate::data::paths;
use crate::macguiver::platform::Platform;
pub use cores::CoreManager;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum TopLevelViewType {
    #[default]
    MainMenu,
    Cores,
    Settings,
    InputTester,
    About,
    Quit,
}

impl MenuReturn for TopLevelViewType {
    fn back() -> Self {
        Self::MainMenu
    }
}

impl TopLevelViewType {
    pub fn function(&self) -> Option<fn(&mut MiSTer) -> TopLevelViewType> {
        match self {
            TopLevelViewType::MainMenu => Some(main_menu),
            TopLevelViewType::About => None,
            TopLevelViewType::Settings => Some(settings_panel),
            TopLevelViewType::Cores => Some(cores_menu_panel),
            TopLevelViewType::InputTester => Some(input_tester),
            TopLevelViewType::Quit => None,
        }
    }
}

pub struct MiSTer {
    toolbar: Toolbar,
    settings: Arc<Settings>,
    database: Arc<RwLock<Connection>>,
    view: TopLevelViewType,

    render_toolbar: bool,

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
        let database = Arc::new(RwLock::new(database));
        let toolbar_size = platform.toolbar_dimensions();
        let main_size = platform.main_dimensions();
        let core_manager = CoreManager::new(database.clone());

        Self {
            toolbar: Toolbar::new(settings.clone(), database.clone()),
            view: TopLevelViewType::default(),
            render_toolbar: true,
            core_manager: Arc::new(RwLock::new(core_manager)),
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

        self.event_loop(|app, _state| match app.view.function() {
            None => Some(TopLevelViewType::Quit),
            Some(f) => {
                app.view = f(app);
                None
            }
        });
    }

    fn main_buffer(&mut self) -> &mut DrawBuffer<Self::Color> {
        &mut self.main_buffer
    }

    fn database(&self) -> Arc<RwLock<Connection>> {
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
        // Due to a limitation in Rust language right now, None does not implement Copy
        // when Option<T> does not. This means we can't use it in an array. So we use a
        // constant to work around this.
        const NONE: Option<Joystick> = None;
        let mut joysticks: [Option<Joystick>; 16] = [NONE; 16];

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
                        joysticks[*which as usize] = Some(j);
                    }
                    Event::JoyDeviceRemoved { which, .. } => {
                        joysticks[*which as usize] = None;
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
