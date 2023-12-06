use crate::data::paths;
use bus::{Bus, BusReader};
use merge::Merge;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;
use strum::Display;
use tracing::{debug, error};

pub mod mappings;
use mappings::MappingSettings;

fn default_retronomicon_backend_() -> Vec<Url> {
    vec![Url::parse("https://retronomicon.land/api/v1/").unwrap()]
}

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
    true
}

fn invert_toolbar_() -> bool {
    true
}

#[derive(Default, Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Display)]
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
    pub fn next(&self) -> Self {
        match self {
            DateTimeFormat::Default => DateTimeFormat::Short,
            DateTimeFormat::Short => DateTimeFormat::TimeOnly,
            DateTimeFormat::TimeOnly => DateTimeFormat::Hidden,
            DateTimeFormat::Hidden => DateTimeFormat::Default,
        }
    }

    pub fn time_format(&self) -> String {
        match self {
            DateTimeFormat::Default => "%c".to_string(),
            DateTimeFormat::Hidden => "".to_string(),
            DateTimeFormat::Short => "%x %X".to_string(),
            DateTimeFormat::TimeOnly => "%X".to_string(),
        }
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, Merge)]
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
    mappings: MappingSettings,

    #[serde(default)]
    #[merge(strategy = merge::overwrite)]
    language: Option<String>,
}

impl Default for InnerSettings {
    fn default() -> Self {
        Self {
            show_fps: false,
            invert_toolbar: true,
            toolbar_datetime_format: DateTimeFormat::default(),
            mappings: MappingSettings::default(),
            language: None,
        }
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

    pub fn mappings(&self) -> &MappingSettings {
        &self.mappings
    }

    pub fn mappings_mut(&mut self) -> &mut MappingSettings {
        &mut self.mappings
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
        Self::default_with_inner(InnerSettings::default())
    }
}

impl Merge for Settings {
    fn merge(&mut self, other: Self) {
        *self.path.write().unwrap() = other.path.read().unwrap().to_path_buf();
        self.inner
            .write()
            .unwrap()
            .merge(other.inner.read().unwrap().clone());
    }
}

impl Settings {
    fn default_with_inner(inner: InnerSettings) -> Self {
        let mut update = Bus::new(1);

        let inner = Arc::new(RwLock::new(inner));
        let path = Arc::new(RwLock::new(PathBuf::new()));
        let drop_send = create_settings_save_thread_(update.add_rx(), path.clone(), inner.clone());

        Self {
            path,
            inner,
            update: RefCell::new(update),
            drop_send,
        }
    }

    pub fn update(&self, other: InnerSettings) {
        if self.inner.write().unwrap().merge_check(other) {
            let _ = self.update.borrow_mut().try_broadcast(());
        }
    }

    pub fn update_done(&self) {
        let _ = self.update.borrow_mut().try_broadcast(());
    }

    /// Load the settings from disk.
    pub fn new() -> Self {
        let mut settings: Option<Self> = None;

        for path in paths::all_settings_paths() {
            if let Some(s) = settings.as_mut() {
                let other = Self::load(&path).unwrap().inner_mut().clone();
                eprintln!("1 Loaded settings from {:?}: {:#?}", path, other);
                s.inner_mut().merge(other);
            } else if let Ok(s) = Self::load(&path) {
                eprintln!("2 Loaded settings from {:?}: {:#?}", path, s);
                settings = Some(s);
            }
        }

        debug!(?settings, "Settings loaded");
        settings.unwrap_or_default()
    }

    fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path.as_ref())?;
        // Parsing errors are ignored.
        match json5::from_str(&content) {
            Ok(settings) => Ok(Self::default_with_inner(settings)),
            Err(e) => {
                error!(
                    "Failed to parse settings at path {:?}: {}",
                    path.as_ref(),
                    e
                );
                error!("This is not an error, the settings file will be ignored.");
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to parse settings",
                ))
            }
        }
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
    pub fn toggle_show_fps(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.show_fps = !inner.show_fps;
    }

    #[inline]
    pub fn toggle_invert_toolbar(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.invert_toolbar = !inner.invert_toolbar;
    }

    #[inline]
    pub fn toggle_toolbar_datetime_format(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.toolbar_datetime_format = inner.toolbar_datetime_format.next();
    }

    #[inline]
    pub fn set_toolbar_datetime_format(&self, v: DateTimeFormat) {
        self.inner.write().unwrap().toolbar_datetime_format = v;
    }

    #[inline]
    pub fn retronomicon_backend(&self) -> Vec<Url> {
        default_retronomicon_backend_()
    }

    #[inline]
    pub fn inner(&self) -> RwLockReadGuard<'_, InnerSettings> {
        self.inner.read().unwrap()
    }

    #[inline]
    pub fn inner_mut(&self) -> RwLockWriteGuard<'_, InnerSettings> {
        self.inner.write().unwrap()
    }

    #[inline]
    pub fn reset_all_settings(&self) {
        let folders = [
            paths::config_root_path(),
            paths::core_root_path(),
            paths::settings_path(),
        ];
        for f in folders {
            if f.exists() {
                std::fs::remove_dir_all(f).unwrap();
            }
        }

        #[cfg(target_os = "linux")]
        unsafe {
            libc::reboot(libc::RB_AUTOBOOT);
        }
    }
}

#[test]
fn serializes() {
    let mut settings = InnerSettings::default();
    let mut other_serialized = InnerSettings::default();
    other_serialized.mappings.add(
        crate::input::commands::ShortcutCommand::ShowCoreMenu,
        crate::input::Shortcut::default().with_key(sdl3::keyboard::Scancode::A),
    );

    settings.merge(other_serialized);
    let serialized = json5::to_string(&settings).unwrap();
    assert_eq!(
        serialized,
        r#"{"show_fps":false,"invert_toolbar":true,"toolbar_datetime_format":"Default","mappings":{"quit_core":"'F10'","reset_core":"'F11'","show_menu":["'A'","'F12'"],"take_screenshot":"'SysReq'"},"language":null}"#
    );
}
