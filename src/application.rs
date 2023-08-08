use crate::application::toolbar::Toolbar;
use crate::application::widgets::keyboard::KeyboardTesterWidget;
use crate::data::settings::Settings;
use crate::macguiver::application::{Application, UpdateResult};
use crate::macguiver::buffer::DrawBuffer;
use crate::main_inner::Flags;
use crate::platform::{PlatformState, WindowManager};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use sdl3::event::Event;

mod icons;
mod menu;
mod toolbar;
mod widgets;

pub trait Panel {
    fn new(settings: &Settings) -> Self
    where
        Self: Sized;
    fn update(&mut self, state: &PlatformState) -> Result<Option<TopLevelViewType>, String>;
    fn draw(&self, target: &mut DrawBuffer<BinaryColor>);
}

pub struct KeyboardTesterView {
    widget: KeyboardTesterWidget,
}

impl Panel for KeyboardTesterView {
    fn new(_settings: &Settings) -> Self {
        Self {
            widget: KeyboardTesterWidget::new(),
        }
    }

    fn update(&mut self, state: &PlatformState) -> Result<Option<TopLevelViewType>, String> {
        let mut should_next = false;

        for event in state.events() {
            match event {
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if *keycode == sdl3::keyboard::Keycode::Tab {
                        should_next = true;
                    }
                    self.widget.insert((*keycode).into());
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    self.widget.remove((*keycode).into());
                }
                _ => {}
            }
        }

        if should_next {
            Ok(Some(TopLevelViewType::MainMenu))
        } else {
            Ok(None)
        }
    }

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
        self.widget.draw(target).unwrap();
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TopLevelViewType {
    KeyboardTester,
    IconView,
    MainMenu,
}

/// Top-level Views for the MiSTer application.
pub struct TopLevelView {
    ty: TopLevelViewType,
    panel: Box<dyn Panel>,
    settings: Settings,
}

impl TopLevelView {
    pub fn next(&mut self, ty: TopLevelViewType) {
        if self.ty != ty {
            self.panel = match ty {
                TopLevelViewType::KeyboardTester => {
                    Box::new(KeyboardTesterView::new(&self.settings))
                }
                TopLevelViewType::IconView => Box::new(icons::IconView::new(&self.settings)),
                TopLevelViewType::MainMenu => Box::new(menu::MainMenu::new(&self.settings)),
            };
            self.ty = ty;
        }
    }
}

impl Panel for TopLevelView {
    fn new(settings: &Settings) -> Self {
        Self {
            ty: TopLevelViewType::MainMenu,
            panel: Box::new(menu::MainMenu::new(settings)),
            settings: settings.clone(),
        }
    }

    fn update(&mut self, state: &PlatformState) -> Result<Option<TopLevelViewType>, String> {
        self.panel.update(state)
    }

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
        self.panel.draw(target);
    }
}

pub struct MiSTer {
    toolbar: Toolbar,
    settings: Settings,
    view: TopLevelView,
}

impl MiSTer {
    pub fn new() -> Self {
        let settings = Settings::new();

        Self {
            toolbar: Toolbar::new(&settings),
            view: TopLevelView::new(&settings),
            settings,
        }
    }

    pub fn run(&mut self, flags: Flags) -> Result<(), String> {
        let mut window_manager = WindowManager::default();
        window_manager.run(self, flags)
    }
}

impl Application for MiSTer {
    type Color = BinaryColor;

    fn settings(&self) -> &Settings {
        &self.settings
    }

    fn update(&mut self, state: &PlatformState) -> UpdateResult {
        if state.events().any(|ev| matches!(ev, Event::Quit { .. })) {
            return UpdateResult::Quit;
        }

        let should_redraw_toolbar = self.toolbar.update();
        match self.view.update(state) {
            Ok(Some(next_view)) => {
                self.view.next(next_view);
            }
            Ok(None) => {}
            Err(e) => panic!("{}", e),
        };

        UpdateResult::Redraw(should_redraw_toolbar, true)
    }

    fn draw_title(&self, target: &mut DrawBuffer<BinaryColor>) {
        self.toolbar.draw(target).unwrap();
    }

    fn draw_main(&self, target: &mut DrawBuffer<BinaryColor>) {
        self.view.draw(target);
    }
}
