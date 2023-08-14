use crate::application::panels::input_tester::input_tester;
use crate::application::toolbar::Toolbar;
use crate::data::settings::Settings;
use crate::macguiver::application::{Application, EventLoopState};
use crate::macguiver::buffer::DrawBuffer;
use crate::main_inner::Flags;
use crate::platform::{MiSTerPlatform, WindowManager};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use sdl3::event::Event;
use std::sync::{Arc, RwLock};

// mod icons;
// pub mod menu;

mod panels;
mod toolbar;
mod widgets;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TopLevelViewType {
    // MainMenu,
    // Settings,
    KeyboardTester,
    Quit,
}

impl TopLevelViewType {
    pub fn function(&self) -> Option<fn(&mut MiSTer) -> TopLevelViewType> {
        match self {
            TopLevelViewType::KeyboardTester => Some(panels::input_tester::input_tester),
            TopLevelViewType::Quit => None,
        }
    }
}

pub struct MiSTer {
    toolbar: Toolbar,
    settings: Settings,
    database: Arc<RwLock<mister_db::Connection>>,
    view: TopLevelViewType,

    pub platform: WindowManager,
    main_buffer: DrawBuffer<BinaryColor>,
    toolbar_buffer: DrawBuffer<BinaryColor>,
}

impl MiSTer {
    pub fn new(platform: WindowManager) -> Self {
        let settings = Settings::new();
        let database = mister_db::establish_connection();
        let database = Arc::new(RwLock::new(database));
        let toolbar_size = platform.toolbar_dimensions();
        let main_size = platform.main_dimensions();

        Self {
            toolbar: Toolbar::new(&settings, database.clone()),
            view: TopLevelViewType::KeyboardTester,
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

    fn settings(&self) -> &Settings {
        &self.settings
    }

    fn run(&mut self, _flags: Flags) {
        self.event_loop(|app, state| match app.view {
            TopLevelViewType::KeyboardTester => {
                app.view = input_tester(app);
                None
            }
            TopLevelViewType::Quit => Some(TopLevelViewType::Quit),
        });
    }

    fn main_buffer(&mut self) -> &mut DrawBuffer<Self::Color> {
        &mut self.main_buffer
    }

    fn event_loop(
        &mut self,
        mut loop_fn: impl FnMut(&mut Self, &mut EventLoopState) -> Option<TopLevelViewType>,
    ) -> TopLevelViewType {
        loop {
            let events = self.platform.events();
            for event in events.iter() {
                if let Event::Quit { .. } = event {
                    return TopLevelViewType::Quit;
                }
            }

            let mut state = EventLoopState::new(events);

            if let Some(r) = loop_fn(self, &mut state) {
                break r;
            }

            if self.toolbar.update() {
                self.toolbar_buffer.clear(BinaryColor::Off).unwrap();
                self.toolbar.draw(&mut self.toolbar_buffer).unwrap();
            }

            self.platform.update_main(&self.main_buffer);
            self.platform.update_toolbar(&self.toolbar_buffer);
        }
    }
}
