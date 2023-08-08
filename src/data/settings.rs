use merge::Merge;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn show_fps_default_() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize, Merge)]
struct InnerSettings {
    #[serde(default = "show_fps_default_")]
    #[merge(strategy = merge::overwrite)]
    show_fps: bool,
}

impl Default for InnerSettings {
    fn default() -> Self {
        Self { show_fps: true }
    }
}

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
        let file = std::fs::File::open(path)?;
        let settings: InnerSettings = serde_json::from_reader(file)?;
        let (update_send, update_recv) = crossbeam_channel::unbounded();

        Ok(Self {
            inner: Rc::new(RefCell::new(settings)),
            update_send,
            update_recv,
        })
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
