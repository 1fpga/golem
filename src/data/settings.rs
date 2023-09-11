use crate::application::menu::style::{menu_style, MenuReturn, SdlMenuInputAdapter};
use crate::application::TopLevelViewType;
use crate::data::paths;
use bus::{Bus, BusReader};
use embedded_menu::selection_indicator::style::Invert;
use embedded_menu::selection_indicator::AnimatedPosition;
use embedded_menu::{Menu, SelectValue};
use merge::Merge;
use sdl3::keyboard::Keycode;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{debug, error};

fn create_settings_save_thread_(
    mut update_recv: BusReader<()>,
    _path: Arc<RwLock<PathBuf>>,
    inner: Arc<RwLock<InnerSettings>>,
) -> crossbeam_channel::Sender<()> {
    let (drop_send, drop_recv) = crossbeam_channel::bounded(1);
    let debouncer = debounce::thread::EventDebouncer::new(Duration::from_millis(500), move |_| {
        // let path = path.read().unwrap();
        let path = paths::settings_path();
        if let Err(e) = inner.read().unwrap().save(path.as_path()) {
            // Still ignore error. Maybe filesystem is readonly?
            error!("Failed to save settings: {}", e);
        }
    });

    std::thread::spawn(move || loop {
        if update_recv.recv_timeout(Duration::from_secs(1)).is_ok() {
            debouncer.put(());
        }
        if drop_recv.try_recv().is_ok() {
            break;
        }
    });

    drop_send
}

fn show_fps_default_() -> bool {
    false
}

fn invert_toolbar_() -> bool {
    true
}

#[derive(Default, Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, SelectValue)]
pub enum DateTimeFormat {
    /// The default local format for datetime (respecting Locale).
    #[default]
    Default,

    /// Short locale format.
    Short,

    /// Only show the time.
    TimeOnly,

    /// Hide the datetime.
    Hidden,
}

impl DateTimeFormat {
    pub fn time_format(&self) -> String {
        match self {
            DateTimeFormat::Default => "%c".to_string(),
            DateTimeFormat::Hidden => "".to_string(),
            DateTimeFormat::Short => "%x %X".to_string(),
            DateTimeFormat::TimeOnly => "%X".to_string(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, SelectValue)]
pub enum MenuKeyBinding {
    /// The default local format for datetime (respecting Locale).
    #[default]
    F12,

    /// Short locale format.
    F11,

    /// Only show the time.
    PrtSc,
}

impl PartialEq<Keycode> for MenuKeyBinding {
    fn eq(&self, other: &Keycode) -> bool {
        matches!(
            (self, other),
            (Self::F12, Keycode::F12)
                | (Self::F11, Keycode::F11)
                | (Self::PrtSc, Keycode::PrintScreen)
        )
    }
}

#[derive(Debug, Copy, Clone, Hash, Serialize, Deserialize, Merge, Menu)]
#[menu(
    title = "Settings",
    navigation(events = TopLevelViewType, marker = ""),
    items = [
        data(label = "Show FPS", field = show_fps),
        data(label = "Invert toolbar colors", field = invert_toolbar),
        data(label = "Toolbar date format", field = toolbar_datetime_format),
        data(label = "Menu Key Binding", field = menu_key_bind),
        navigation(label = "Back", event = TopLevelViewType::MainMenu)
    ]
)]
pub struct InnerSettings {
    #[serde(default = "show_fps_default_")]
    #[merge(strategy = merge::overwrite)]
    show_fps: bool,

    #[serde(default = "invert_toolbar_")]
    #[merge(strategy = merge::overwrite)]
    invert_toolbar: bool,

    #[serde(default)]
    #[merge(strategy = merge::overwrite)]
    toolbar_datetime_format: DateTimeFormat,

    #[serde(default)]
    #[merge(strategy = merge::overwrite)]
    menu_key_bind: MenuKeyBinding,
}

impl Default for InnerSettings {
    fn default() -> Self {
        Self {
            show_fps: false,
            invert_toolbar: true,
            toolbar_datetime_format: DateTimeFormat::default(),
            menu_key_bind: MenuKeyBinding::F12,
        }
    }
}

impl MenuReturn for InnerSettingsMenuEvents {
    fn back() -> Self {
        Self::NavigationEvent(TopLevelViewType::back())
    }
}

impl InnerSettings {
    /// Merge this with another settings object, returning true if any changes were made.
    pub(super) fn merge_check(&mut self, other: InnerSettings) -> bool {
        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        let old = hasher.finish();

        self.merge(other);

        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        old != hasher.finish()
    }

    pub(super) fn save(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let mut path = path.as_ref().to_path_buf();
        if path.as_os_str().is_empty() {
            path = paths::config_root_path().join("settings.json5");
        }

        debug!("Saving settings to {path:?}");
        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // TODO: find a way to keep comments from the file.
        let content = json5format::format(
            &json5::to_string(self).unwrap(),
            Some(path.to_string_lossy().to_string()),
            None,
        )
        .unwrap();

        std::fs::write(path, content)
    }
}

#[derive(Debug)]
pub struct Settings {
    path: Arc<RwLock<PathBuf>>,
    inner: Arc<RwLock<InnerSettings>>,
    update: RefCell<Bus<()>>,
    drop_send: crossbeam_channel::Sender<()>,
}

impl Drop for Settings {
    fn drop(&mut self) {
        let _ = self.drop_send.send(());
    }
}

impl Default for Settings {
    fn default() -> Self {
        let mut update = Bus::new(1);

        let inner = Arc::new(RwLock::new(InnerSettings::default()));
        let path = Arc::new(RwLock::new(PathBuf::new()));
        let drop_send = create_settings_save_thread_(update.add_rx(), path.clone(), inner.clone());

        Self {
            path,
            inner,
            update: RefCell::new(update),
            drop_send,
        }
    }
}

impl Merge for Settings {
    fn merge(&mut self, other: Self) {
        *self.path.write().unwrap() = other.path.read().unwrap().to_path_buf();
        self.inner
            .write()
            .unwrap()
            .merge(*other.inner.read().unwrap());
    }
}

impl Settings {
    pub fn menu(
        &self,
    ) -> InnerSettingsMenuWrapper<
        SdlMenuInputAdapter<InnerSettingsMenuEvents>,
        AnimatedPosition,
        Invert,
    > {
        self.inner
            .read()
            .unwrap()
            .create_menu_with_style(menu_style())
    }

    pub fn update(&self, other: InnerSettings) {
        if self.inner.write().unwrap().merge_check(other) {
            self.update.borrow_mut().broadcast(());
        }
    }

    /// Load the settings from disk.
    pub fn new() -> Self {
        let mut settings = Self::default();

        for path in paths::all_settings_paths() {
            settings.load(path).unwrap();
        }

        settings
    }

    fn load(&mut self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let settings: InnerSettings = json5::from_str(&content).expect("Failed to parse settings");
        self.inner.write().unwrap().merge(settings);
        Ok(())
    }

    pub fn on_update(&self) -> BusReader<()> {
        self.update.borrow_mut().add_rx()
    }

    #[inline]
    pub fn show_fps(&self) -> bool {
        self.inner.read().unwrap().show_fps
    }

    #[inline]
    pub fn invert_toolbar(&self) -> bool {
        self.inner.read().unwrap().invert_toolbar
    }

    #[inline]
    pub fn toolbar_datetime_format(&self) -> DateTimeFormat {
        self.inner.read().unwrap().toolbar_datetime_format
    }

    #[inline]
    pub fn menu_key_binding(&self) -> MenuKeyBinding {
        self.inner.read().unwrap().menu_key_bind
    }
}
