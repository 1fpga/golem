use std::any::Any;
use std::cell::UnsafeCell;
use std::io::{Read, Seek, Write};
use std::rc::Rc;
use std::time::SystemTime;

pub use bios::Bios;
use image::DynamicImage;
pub use null::NullCore;
pub use rom::Rom;
use serde::Serialize;

use crate::inputs::{gamepad, keyboard};

pub mod bios;
pub mod null;
pub mod rom;

/// An ID that is given by the core implementation for a config menu. This
/// ID is used to identify the menu item that was selected by the user.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SettingId(u32);

impl Serialize for SettingId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl From<u32> for SettingId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<SettingId> for u32 {
    fn from(value: SettingId) -> Self {
        value.0
    }
}

impl From<&str> for SettingId {
    fn from(value: &str) -> Self {
        Self::from_label(value)
    }
}

impl From<&String> for SettingId {
    fn from(value: &String) -> Self {
        Self::from_label(value)
    }
}

impl SettingId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn from_label(label: &str) -> Self {
        let mut s: u32 = 0;
        for c in label.as_bytes() {
            s = s.wrapping_mul(223).wrapping_add(*c as u32);
        }
        Self(s)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// Core Settings, which is a list of setting items that can be displayed
/// in the core's setting menu. This is an abstraction over all possible
/// settings that a core can have.
#[derive(Debug, Serialize)]
pub struct CoreSettings {
    title: String,
    items: Vec<CoreSettingItem>,
}

impl CoreSettings {
    pub fn new(title: String, items: Vec<CoreSettingItem>) -> Self {
        Self { title, items }
    }

    pub fn items(&self) -> &[CoreSettingItem] {
        &self.items
    }
}

/// A core setting item that can be displayed in the core's setting menu.
/// This is used to configure the core's settings, in an abstract way.
///
/// Please note that a core can create menu items that don't have actual
/// effect on the core's behavior.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum CoreSettingItem {
    /// A menu page that contains more menu items. This is purely cosmetic.
    Page {
        /// A unique ID for this page.
        id: SettingId,

        /// The label that would be shown in a top-level menu.
        label: String,

        /// The title of this menu. Can be different than label.
        title: String,

        /// Menu Items contained by this menu.
        items: Vec<CoreSettingItem>,

        /// Whether this page is disabled from being selected.
        disabled: bool,
    },

    /// A separator (horizontal line normally). Cosmetic.
    Separator,

    /// A simple label that might be selectable, but is not considered
    /// actionable. This is used to display information to the user.
    Label {
        selectable: bool,
        label: String,
        disabled: bool,
    },

    /// A file select menu item that can be used to select a file from the
    /// file system. This is used to load files into the core.
    #[serde(rename = "file")]
    FileSelect {
        id: SettingId,
        label: String,
        extensions: Vec<String>,
        disabled: bool,
    },

    /// A trigger that can be used to perform an action.
    Trigger {
        id: SettingId,
        label: String,
        disabled: bool,
    },

    /// An option that can be selected by the user and contains a boolean
    /// value (on or off).
    #[serde(rename = "bool")]
    BoolOption {
        id: SettingId,
        label: String,
        value: bool,
        disabled: bool,
    },

    /// An option that can be selected by the user and contains an integer
    /// value. This is used for options that have a range of values, but
    /// can also represent options with 2 choices.
    ///
    /// It is an error to have choices less than 2 items (and will result
    /// in an error when dealing with core menus).
    #[serde(rename = "int")]
    IntOption {
        id: SettingId,
        label: String,
        choices: Vec<String>,
        value: usize,
        disabled: bool,
    },
}

impl CoreSettingItem {
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.set_disable(disabled);
        self
    }

    pub fn set_disable(&mut self, new_disabled: bool) {
        match self {
            CoreSettingItem::Page { disabled, .. }
            | CoreSettingItem::Label { disabled, .. }
            | CoreSettingItem::Trigger { disabled, .. }
            | CoreSettingItem::FileSelect { disabled, .. }
            | CoreSettingItem::BoolOption { disabled, .. }
            | CoreSettingItem::IntOption { disabled, .. } => {
                *disabled = new_disabled;
            }
            _ => {}
        }
    }

    pub fn page(
        id: impl Into<SettingId>,
        label: &str,
        title: &str,
        items: Vec<CoreSettingItem>,
    ) -> Self {
        CoreSettingItem::Page {
            id: id.into(),
            label: label.to_string(),
            title: title.to_string(),
            items,
            disabled: false,
        }
    }

    pub fn items(&self) -> Option<&Vec<CoreSettingItem>> {
        match self {
            CoreSettingItem::Page { items, .. } => Some(items),
            _ => None,
        }
    }

    pub fn items_mut(&mut self) -> Option<&mut Vec<CoreSettingItem>> {
        match self {
            CoreSettingItem::Page { items, .. } => Some(items),
            _ => None,
        }
    }

    pub fn separator() -> Self {
        CoreSettingItem::Separator
    }

    pub fn label(selectable: bool, label: &str) -> Self {
        CoreSettingItem::Label {
            selectable,
            label: label.to_string(),
            disabled: false,
        }
    }

    pub fn file_select(id: impl Into<SettingId>, label: &str, extensions: Vec<String>) -> Self {
        CoreSettingItem::FileSelect {
            id: id.into(),
            label: label.to_string(),
            extensions,
            disabled: false,
        }
    }

    pub fn trigger(id: impl Into<SettingId>, label: &str) -> Self {
        CoreSettingItem::Trigger {
            id: id.into(),
            label: label.to_string(),
            disabled: false,
        }
    }

    pub fn bool_option(id: impl Into<SettingId>, label: &str, value: Option<bool>) -> Self {
        CoreSettingItem::BoolOption {
            id: id.into(),
            label: label.to_string(),
            value: value.unwrap_or_default(),
            disabled: false,
        }
    }

    pub fn int_option(
        id: impl Into<SettingId>,
        label: &str,
        choices: Vec<String>,
        value: Option<usize>,
    ) -> Self {
        CoreSettingItem::IntOption {
            id: id.into(),
            label: label.to_string(),
            choices,
            value: value.unwrap_or_default(),
            disabled: false,
        }
    }

    pub fn add_item(&mut self, sub: CoreSettingItem) {
        if let CoreSettingItem::Page { items, .. } = self {
            items.push(sub);
        }
    }
}

/// A save state, if cores support it. This is used to save the state of the
/// internal memory of the core, so that it can be restored later.
pub trait SaveState {
    /// Returns true if the save state is dirty and needs to be saved.
    fn is_dirty(&self) -> bool;

    /// Save the state of the core to a buffer.
    fn save(&mut self, writer: &mut dyn Write) -> Result<(), Error>;

    /// Load the state of the core from a buffer.
    fn load(&mut self, reader: &mut dyn Read) -> Result<(), Error>;
}

/// A mounted file to the core. This could be a SAV file, a hard drive, a memory card, or
/// any other kind of file that can be mounted to the core.
pub trait MountedFile: Read + Write + Seek {}

/// An error that can be returned by a core.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An error occurred: {0}")]
    Generic(String),

    #[error("An error occurred: {0}")]
    IoError(#[from] std::io::Error),

    #[error("An error occurred: {0}")]
    Message(String),

    #[error("An error occurred: {0}")]
    AnyError(#[from] Box<dyn std::error::Error>),
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::Generic(value)
    }
}

/// An iterator over the save states of a core.
#[derive(Copy, Clone)]
pub struct SaveStateIter<'a, T: Core> {
    core: &'a T,
    valid: bool,
    index: usize,
}

impl<'a, T: Core> SaveStateIter<'a, T> {
    pub fn new(core: &'a T) -> Self {
        Self {
            core,
            valid: true,
            index: 0,
        }
    }
}

impl<'a, T: Core> Iterator for SaveStateIter<'a, T> {
    type Item = &'a dyn SaveState;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.valid {
            return None;
        }

        match self.core.save_state(self.index) {
            Ok(Some(state)) => {
                self.index += 1;
                Some(state)
            }
            Ok(None) => None,
            Err(_e) => {
                self.valid = false;
                None
            }
        }
    }
}

pub trait Core {
    /// Initialize the core and anything needed for it to run.
    fn init(&mut self) -> Result<(), Error>;

    /// Return a human-readable name for the core.
    fn name(&self) -> &str;

    /// Reset the core, restarting the ROM from the beginning.
    fn reset(&mut self) -> Result<(), Error>;

    /// Update the volume of a core. This value is to be taken as a linear
    /// scale from 0-256 requested from the user. If the core takes a
    /// logarithmic scale, it should convert it to the appropriate value.
    fn set_volume(&mut self, volume: u8) -> Result<(), Error>;

    /// Sets the time in the core's RTC.
    fn set_rtc(&mut self, time: SystemTime) -> Result<(), Error>;

    /// Take a screenshot (if supported), returning the image.
    fn screenshot(&self) -> Result<DynamicImage, Error>;

    /// Get the save state at a specific slot.
    /// If the core does not support save states, this should return `None` for all slots.
    /// If the core supports save states, but the slot index is out of bound, this should
    /// return `None`.
    fn save_state_mut(&mut self, slot: usize) -> Result<Option<&mut dyn SaveState>, Error>;

    /// Get the save state at a specific slot.
    /// If the core does not support save states, this should return `None` for all slots.
    /// If the core supports save states, but the slot index is out of bound, this should
    /// return `None`.
    fn save_state(&self, slot: usize) -> Result<Option<&dyn SaveState>, Error>;

    /// Get the mounted save file at a specific slot.
    /// If the core does not support mounting files, this should return `None` for all slots.
    /// If the core supports mounting files, but the slot index is out of bound, this should
    /// return `None`.
    fn mounted_file_mut(&mut self, slot: usize) -> Result<Option<&mut dyn MountedFile>, Error>;

    /// Load a ROM into the core.
    fn send_rom(&mut self, rom: Rom) -> Result<(), Error>;

    /// Load a BIOS into the core. For cores that support multiple BIOS, the BIOS should be
    /// selected based on the core's configuration and the slot information should be part
    /// of [`&dyn Bios`].
    fn send_bios(&mut self, bios: Bios) -> Result<(), Error>;

    /// Send a key up event to the core.
    fn key_up(&mut self, key: keyboard::Scancode) -> Result<(), Error>;

    /// Send a key down event to the core.
    fn key_down(&mut self, key: keyboard::Scancode) -> Result<(), Error>;

    /// Set the keys that are currently pressed, releasing the keys that are not in
    /// the list.
    fn keys_set(&mut self, keys: keyboard::ScancodeSet) -> Result<(), Error>;

    /// Return a slice of the keys that are currently pressed. The order of the
    /// keys in the slice is not guaranteed.
    fn keys(&self) -> Result<keyboard::ScancodeSet, Error>;

    fn gamepad_button_up(&mut self, index: usize, button: gamepad::Button) -> Result<(), Error>;
    fn gamepad_button_down(&mut self, index: usize, button: gamepad::Button) -> Result<(), Error>;
    fn gamepad_buttons_set(
        &mut self,
        index: usize,
        buttons: gamepad::ButtonSet,
    ) -> Result<(), Error>;

    /// Returns the gamepad buttons that are currently pressed. The order of the
    /// buttons in the slice is not guaranteed.
    /// If the core does not support gamepads at the index requested, this should
    /// return `None`.
    fn gamepad_buttons(&self, index: usize) -> Result<Option<gamepad::ButtonSet>, Error>;

    /// Returns the menu items that the core supports. This would correspond to the
    /// top level page of config items. If the core does not support a menu, this
    /// should return an empty vector.
    fn settings(&self) -> Result<CoreSettings, Error>;

    /// Trigger a menu item in the core. This is used to perform an action based on
    /// the menu item selected by the user. It can also be linked to a shortcut.
    fn trigger(&mut self, id: SettingId) -> Result<(), Error>;

    /// Send a file path to the core. This is used to load files into the core.
    fn file_select(&mut self, id: SettingId, path: String) -> Result<(), Error>;

    /// Set an integer option in the core. This is used to set an option that has a
    /// positive integer value. Returns the new value (e.g. if value is out of bound,
    /// the core might clip it or reset it).
    fn int_option(&mut self, id: SettingId, value: u32) -> Result<u32, Error>;

    /// Set a boolean option in the core. This is used to set an option that has a
    /// boolean value that can be toggled by the user. Returns the new value
    /// (e.g. if the core cannot change the value, returns the previous one).
    fn bool_option(&mut self, id: SettingId, value: bool) -> Result<bool, Error>;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Indicates to the core that it needs to prepare quitting.
    /// This is used to perform any cleanup that the core needs to do before quitting.
    /// When the core is ready, [should_quit()] should return true.
    fn quit(&mut self);

    /// Returns true if the core should quit. Some cores might want to quit
    /// back to the main menu by themselves.
    fn should_quit(&self) -> bool;
}

/// A core that be used in the `Golem` platform. This is a wrapper around a core
/// that implements the [`Core`] trait. It can be used to pass around a core
/// without knowing its implementation.
///
/// # Safety
/// We can use UnsafeCell here because we are not sharing the core across threads.
/// Although this can still lead to undefined behaviour, the underlying core still
/// has access to physical memory and this does not make it less safe.
#[derive(Clone)]
pub struct GolemCore {
    name: String,
    inner: Rc<UnsafeCell<dyn Core + 'static>>,
}

impl GolemCore {
    pub fn new(core: impl Core + 'static) -> Self {
        Self {
            name: core.name().to_string(),
            inner: Rc::new(UnsafeCell::new(core)),
        }
    }

    pub fn null() -> Self {
        Self::new(NullCore)
    }
}

impl Core for GolemCore {
    fn init(&mut self) -> Result<(), Error> {
        unsafe { &mut *self.inner.get() }.init()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn reset(&mut self) -> Result<(), Error> {
        unsafe { &mut *self.inner.get() }.reset()
    }

    fn set_volume(&mut self, volume: u8) -> Result<(), Error> {
        unsafe { &mut *self.inner.get() }.set_volume(volume)
    }

    fn set_rtc(&mut self, time: SystemTime) -> Result<(), Error> {
        unsafe { &mut *self.inner.get() }.set_rtc(time)
    }

    fn screenshot(&self) -> Result<DynamicImage, Error> {
        unsafe { &mut *self.inner.get() }.screenshot()
    }

    fn save_state_mut(&mut self, slot: usize) -> Result<Option<&mut dyn SaveState>, Error> {
        unsafe { &mut *self.inner.get() }.save_state_mut(slot)
    }

    fn save_state(&self, slot: usize) -> Result<Option<&dyn SaveState>, Error> {
        unsafe { &mut *self.inner.get() }.save_state(slot)
    }

    fn mounted_file_mut(&mut self, slot: usize) -> Result<Option<&mut dyn MountedFile>, Error> {
        unsafe { &mut *self.inner.get() }.mounted_file_mut(slot)
    }

    fn send_rom(&mut self, rom: Rom) -> Result<(), Error> {
        unsafe { &mut *self.inner.get() }.send_rom(rom)
    }

    fn send_bios(&mut self, bios: Bios) -> Result<(), Error> {
        unsafe { &mut *self.inner.get() }.send_bios(bios)
    }

    fn key_up(&mut self, key: keyboard::Scancode) -> Result<(), Error> {
        unsafe { &mut *self.inner.get() }.key_up(key)
    }

    fn key_down(&mut self, key: keyboard::Scancode) -> Result<(), Error> {
        unsafe { &mut *self.inner.get() }.key_down(key)
    }

    fn keys_set(&mut self, keys: keyboard::ScancodeSet) -> Result<(), Error> {
        unsafe { &mut *self.inner.get() }.keys_set(keys)
    }

    fn keys(&self) -> Result<keyboard::ScancodeSet, Error> {
        unsafe { &mut *self.inner.get() }.keys()
    }

    fn gamepad_button_up(&mut self, index: usize, button: gamepad::Button) -> Result<(), Error> {
        unsafe { &mut *self.inner.get() }.gamepad_button_up(index, button)
    }

    fn gamepad_button_down(&mut self, index: usize, button: gamepad::Button) -> Result<(), Error> {
        unsafe { &mut *self.inner.get() }.gamepad_button_down(index, button)
    }

    fn gamepad_buttons_set(
        &mut self,
        index: usize,
        buttons: gamepad::ButtonSet,
    ) -> Result<(), Error> {
        unsafe { &mut *self.inner.get() }.gamepad_buttons_set(index, buttons)
    }

    fn gamepad_buttons(&self, index: usize) -> Result<Option<gamepad::ButtonSet>, Error> {
        unsafe { &mut *self.inner.get() }.gamepad_buttons(index)
    }

    fn settings(&self) -> Result<CoreSettings, Error> {
        unsafe { &mut *self.inner.get() }.settings()
    }

    fn trigger(&mut self, id: SettingId) -> Result<(), Error> {
        unsafe { &mut *self.inner.get() }.trigger(id)
    }

    fn file_select(&mut self, id: SettingId, path: String) -> Result<(), Error> {
        unsafe { &mut *self.inner.get() }.file_select(id, path)
    }

    fn int_option(&mut self, id: SettingId, value: u32) -> Result<u32, Error> {
        unsafe { &mut *self.inner.get() }.int_option(id, value)
    }

    fn bool_option(&mut self, id: SettingId, value: bool) -> Result<bool, Error> {
        unsafe { &mut *self.inner.get() }.bool_option(id, value)
    }

    fn as_any(&self) -> &dyn Any {
        unsafe { &mut *self.inner.get() }.as_any()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        unsafe { &mut *self.inner.get() }.as_any_mut()
    }

    fn quit(&mut self) {
        unsafe { &mut *self.inner.get() }.quit();
    }

    fn should_quit(&self) -> bool {
        unsafe { &*self.inner.get() }.should_quit()
    }
}
