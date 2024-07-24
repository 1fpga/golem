use crate::application::menu::style::{MenuStyleFontSize, MenuStyleOptions};
use crate::data::paths;
use bus::{Bus, BusReader};
use merge::Merge;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;
use strum::Display;
use tracing::{debug, error};

fn default_retronomicon_backend_() -> Vec<Url> {
    vec![Url::parse("https://retronomicon.land/api/v1/").unwrap()]
}

fn create_settings_save_thread_(
    mut update_recv: BusReader<()>,
    path: Arc<RwLock<PathBuf>>,
    inner: Arc<RwLock<InnerSettings>>,
) -> crossbeam_channel::Sender<()> {
    let (drop_send, drop_recv) = crossbeam_channel::bounded(1);
    let debouncer = debounce::thread::EventDebouncer::new(Duration::from_millis(500), move |_| {
        let path = path.read().unwrap().clone();
        if let Err(e) = inner.read().unwrap().save(path) {
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

#[derive(Default, Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Display)]
pub enum DateTimeFormat {
    /// The default local format for datetime (respecting Locale).
    #[default]
    #[serde(rename = "default", alias = "Default")]
    Default,

    /// Short locale format.
    #[serde(rename = "short", alias = "Short")]
    Short,

    /// Only show the time.
    #[serde(rename = "timeOnly", alias = "TimeOnly", alias = "time")]
    TimeOnly,

    /// Hide the datetime.
    #[serde(rename = "hidden", alias = "Hidden", alias = "off")]
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

#[derive(Debug, Default, Clone, Hash, Serialize, Deserialize, Merge)]
#[serde(rename_all = "camelCase")]
pub struct UiSettings {
    #[merge(strategy = merge::option::overwrite_some)]
    show_fps: Option<bool>,

    #[merge(strategy = merge::option::overwrite_some)]
    invert_toolbar: Option<bool>,

    #[merge(strategy = merge::option::overwrite_some)]
    toolbar_datetime_format: Option<DateTimeFormat>,

    #[merge(strategy = merge::option::overwrite_some)]
    menu_font_size: Option<MenuStyleFontSize>,

    #[merge(strategy = merge::option::overwrite_some)]
    language: Option<String>,
}

#[derive(Debug, Default, Clone, Hash, Serialize, Deserialize, Merge)]
#[serde(rename_all = "camelCase")]
pub struct InnerSettings {
    #[merge(strategy = merge::option::recurse)]
    ui: Option<UiSettings>,

    #[serde(default)]
    #[merge(strategy = merge::overwrite)]
    retronomicon_backend: Vec<Url>,
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
                s.inner_mut().merge(other);
            } else if let Ok(s) = Self::load(&path) {
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
        self.inner
            .read()
            .unwrap()
            .ui
            .as_ref()
            .and_then(|ui| ui.show_fps)
            .unwrap_or_default()
    }

    #[inline]
    pub fn invert_toolbar(&self) -> bool {
        self.inner
            .read()
            .unwrap()
            .ui
            .as_ref()
            .and_then(|ui| ui.invert_toolbar)
            .unwrap_or_default()
    }

    #[inline]
    pub fn toolbar_datetime_format(&self) -> DateTimeFormat {
        self.inner
            .read()
            .unwrap()
            .ui
            .as_ref()
            .and_then(|ui| ui.toolbar_datetime_format)
            .unwrap_or_default()
    }

    #[inline]
    pub fn toggle_show_fps(&self) {
        let mut inner = self.inner.write().unwrap();
        let ui = inner.ui.get_or_insert(UiSettings::default());
        let current = ui.show_fps.unwrap_or_default();
        ui.show_fps = Some(!current);
    }

    #[inline]
    pub fn menu_style(&self) -> MenuStyleOptions {
        let font_size = self
            .inner
            .read()
            .unwrap()
            .ui
            .as_ref()
            .and_then(|ui| ui.menu_font_size)
            .unwrap_or_default();
        MenuStyleOptions { font_size }
    }

    #[inline]
    pub fn toggle_invert_toolbar(&self) {
        let mut inner = self.inner.write().unwrap();
        let ui = inner.ui.get_or_insert(UiSettings::default());
        let current = ui.invert_toolbar.unwrap_or_default();
        ui.invert_toolbar = Some(!current);
    }

    pub fn update_from_json(&self, json: serde_json::Value) -> Result<(), String> {
        debug!(from = ?self.inner.read().unwrap(), ?json, "Update settings");

        let value: InnerSettings = serde_json::from_value(json).map_err(|e| e.to_string())?;
        self.update(value);
        Ok(())
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

    #[inline]
    pub fn as_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self.inner.read().unwrap().deref()).unwrap()
    }
}

#[test]
fn update_from_json() {
    let inner = InnerSettings::default();
    let settings = Settings::default_with_inner(inner);
    assert!(!settings.show_fps());

    settings
        .update_from_json(serde_json::json! {
            {
                "ui": {
                    "showFps": true
                }
            }
        })
        .unwrap();

    assert!(settings.show_fps());
}
