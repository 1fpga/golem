use crate::data::paths;
use bus::{Bus, BusReader};
use merge::Merge;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use strum::Display;
use tracing::{debug, error};

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
    false
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
    #[merge(strategy = merge::overwrite)]
    language: Option<String>,
}

impl Default for InnerSettings {
    fn default() -> Self {
        Self {
            show_fps: false,
            invert_toolbar: true,
            toolbar_datetime_format: DateTimeFormat::default(),
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
            .merge(other.inner.read().unwrap().clone());
    }
}

impl Settings {
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
    pub fn set_show_fps(&self, v: bool) {
        self.inner.write().unwrap().show_fps = v;
    }

    #[inline]
    pub fn set_invert_toolbar(&self, v: bool) {
        self.inner.write().unwrap().invert_toolbar = v;
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
