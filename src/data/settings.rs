use merge::Merge;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn show_fps_default_() -> bool {
    false
}

// #[derive(Copy, Clone)]
// pub enum NavEvents {
//     Back,
// }

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Merge /*Menu*/)]
// #[menu(
//     title = "Settings",
//     navigation(events = NavEvents, marker = ">"),
//     items = [
//         data(label = "Show FPS", field = show_fps),
//         navigation(label = "Back", details = "Exits the demo", event = NavEvents::Back)
//     ]
// )]
pub struct InnerSettings {
    #[serde(default = "show_fps_default_")]
    #[merge(strategy = merge::overwrite)]
    show_fps: bool,
}

impl Default for InnerSettings {
    fn default() -> Self {
        Self { show_fps: true }
    }
}

impl InnerSettings {
    pub fn overwrite(&mut self, other: &InnerSettings) -> bool {
        let mut dirty = false;

        if self.show_fps != other.show_fps {
            self.show_fps = other.show_fps;
            dirty = true;
        }

        dirty
    }
}

// type BoxedUpdateFn = Box<dyn FnMut(&PlatformState) -> Result<Option<NavEvents>, String>>;
// type BoxedDrawFn = Box<dyn Fn(&mut DrawBuffer<BinaryColor>)>;
//
// pub struct SettingsPanel {
//     settings: Settings,
//
//     // Function to update the menu.
//     update: BoxedUpdateFn,
//
//     // Function to draw the menu.
//     draw: BoxedDrawFn,
// }
//
// impl Panel for SettingsPanel {
//     fn new(settings: &Settings, _database: Arc<RwLock<mister_db::Connection>>) -> Self {
//         let settings = settings.clone();
//         let menu = settings.inner.borrow().create_menu_with_style(
//             MenuStyle::new(BinaryColor::On)
//                 .with_details_delay(250)
//                 .with_animated_selection_indicator(10)
//                 .with_selection_indicator(AnimatedTriangle::new(200)),
//         );
//
//         let menu = Rc::new(RefCell::new(menu));
//         let (update, draw) = {
//             let menu_update = menu.clone();
//             let menu_draw = menu.clone();
//             let settings = settings.clone();
//
//             let update = move |state: &PlatformState| {
//                 let mut menu = menu_update.borrow_mut();
//                 menu.update(state);
//
//                 let mut result = Ok(None);
//                 for ev in state.events() {
//                     if let Event::KeyDown {
//                         keycode: Some(code),
//                         ..
//                     } = ev
//                     {
//                         match code {
//                             Keycode::Escape => {
//                                 result = Ok(Some(NavEvents::Back));
//                             }
//                             Keycode::Return => {
//                                 result = Ok(menu.interact(InteractionType::Select));
//                             }
//                             Keycode::Up => {
//                                 result = Ok(menu.interact(InteractionType::Previous));
//                             }
//                             Keycode::Down => {
//                                 result = Ok(menu.interact(InteractionType::Next));
//                             }
//                             Keycode::Right => {
//                                 for _ in 0..9 {
//                                     menu.interact(InteractionType::Next);
//                                 }
//                                 result = Ok(menu.interact(InteractionType::Next));
//                             }
//                             _ => {}
//                         }
//                     }
//                 }
//
//                 settings.inner.borrow_mut().overwrite(&menu.data());
//
//                 result
//             };
//
//             let draw = move |target: &mut DrawBuffer<BinaryColor>| {
//                 let menu = menu_draw.borrow();
//                 menu.draw(target).unwrap();
//             };
//
//             (update, draw)
//         };
//
//         Self {
//             settings,
//             update: Box::new(update),
//             draw: Box::new(draw),
//         }
//     }
//
//     fn update(&mut self, state: &PlatformState) -> Result<Option<TopLevelViewType>, String> {
//         let action = (self.update)(state)?;
//         if let Some(action) = action {
//             match action {
//                 NavEvents::Back => Ok(Some(TopLevelViewType::MainMenu)),
//             }
//         } else {
//             self.settings.update_send.send(()).unwrap();
//
//             Ok(None)
//         }
//     }
//
//     fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
//         (self.draw)(target)
//     }
// }

#[derive(Clone)]
pub struct Settings {
    inner: Rc<RefCell<InnerSettings>>,
    update_send: crossbeam_channel::Sender<()>,
    update_recv: crossbeam_channel::Receiver<()>,
}

impl Debug for Settings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Settings")
            .field("show_fps", &self.show_fps())
            .finish()
    }
}

impl Default for Settings {
    fn default() -> Self {
        let (update_send, update_recv) = crossbeam_channel::unbounded();
        Self {
            inner: Rc::new(RefCell::new(InnerSettings::default())),
            update_send,
            update_recv,
        }
    }
}

impl Merge for Settings {
    fn merge(&mut self, other: Self) {
        self.inner.borrow_mut().merge(other.inner.borrow().clone());
    }
}

impl Settings {
    pub fn find_settings_paths() -> Vec<PathBuf> {
        let mut paths = vec![];

        if let Some(mut home) = dirs::home_dir() {
            home.push(".mister");
            home.push("settings.json");

            if home.exists() {
                paths.push(home);
            }
        }
        if let Some(mut exec_settings) = dirs::executable_dir() {
            exec_settings.push("settings.json");

            if exec_settings.exists() {
                paths.push(exec_settings);
            }
        }

        paths
    }

    /// Load the settings from disk.
    pub fn new() -> Self {
        let mut settings = Self::default();

        for path in Self::find_settings_paths() {
            if let Ok(setting) = Self::load(path) {
                settings.merge(setting);
            }
        }

        settings
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let settings: InnerSettings = json5::from_str(&content).expect("Failed to parse settings");
        let (update_send, update_recv) = crossbeam_channel::unbounded();

        Ok(Self {
            inner: Rc::new(RefCell::new(settings)),
            update_send,
            update_recv,
        })
    }

    pub fn overwrite(&mut self, other: &Settings) {
        if self.inner.borrow_mut().overwrite(&other.inner.borrow()) {
            let _ = self.update_send.send(());
        }
    }

    pub fn on_update(&self) -> crossbeam_channel::Receiver<()> {
        self.update_recv.clone()
    }

    pub fn show_fps(&self) -> bool {
        self.inner.borrow().show_fps
    }

    pub fn toggle_fps(&mut self) {
        let mut inner = self.inner.borrow_mut();
        inner.show_fps = !inner.show_fps;
        let _ = self.update_send.send(());
    }
}
