use std::any::Any;
use std::io::{Read, Seek, Write};
use std::time::SystemTime;

use image::DynamicImage;

pub use bios::Bios;
pub use null::NullCore;
pub use rom::Rom;

use crate::inputs::{gamepad, keyboard};

pub mod bios;
pub mod null;
pub mod rom;

/// An ID that is given by the core implementation for a config menu. This
/// ID is used to identify the menu item that was selected by the user.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ConfigMenuId(u32);

/// A menu item that can be displayed in the core's menu.
/// This is used to configure the core's settings, in an abstract
/// way.
///
/// Please note that a core can create menu items that don't have actual
/// effect on the core's behavior.
pub enum CoreMenuItem {
    /// A menu page that contains more menu items. This is purely cosmetic.
    Page {
        sort: i32,
        label: String,
        title: String,
        items: Vec<CoreMenuItem>,
    },

    /// A separator (horizontal line normally). Cosmetic.
    Separator,

    /// A simple label that might be selectable, but is not considered
    /// actionable. This is used to display information to the user.
    Label { selectable: bool, label: String },

    /// A trigger that can be used to perform an action.
    Trigger { id: ConfigMenuId, label: String },

    /// An option that can be selected by the user and contains an integer
    /// value. Booleans are represented as integers with 0 being false and
    /// 1 being true.
    IntOption {
        id: ConfigMenuId,
        label: String,
        choices: Vec<String>,
        value: u32,
    },
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
    fn keys_set(&mut self, keys: &[keyboard::Scancode]) -> Result<(), Error>;

    /// Update the state of the keys.
    fn keys_update(
        &mut self,
        up: &[keyboard::Scancode],
        down: &[keyboard::Scancode],
    ) -> Result<(), Error> {
        for key in up {
            self.key_up(*key)?;
        }
        for key in down {
            self.key_down(*key)?;
        }
        Ok(())
    }

    /// Return a slice of the keys that are currently pressed. The order of the
    /// keys in the slice is not guaranteed.
    fn keys(&self) -> Result<&[keyboard::Scancode], Error>;

    fn gamepad_button_up(&mut self, index: usize, button: gamepad::Button) -> Result<(), Error>;
    fn gamepad_button_down(&mut self, index: usize, button: gamepad::Button) -> Result<(), Error>;
    fn gamepad_buttons_set(
        &mut self,
        index: usize,
        buttons: &[gamepad::Button],
    ) -> Result<(), Error>;
    fn gamepad_buttons_update(
        &mut self,
        index: usize,
        up: &[gamepad::Button],
        down: &[gamepad::Button],
    ) -> Result<(), Error> {
        for button in up {
            self.gamepad_button_up(index, *button)?;
        }
        for button in down {
            self.gamepad_button_down(index, *button)?;
        }
        Ok(())
    }

    /// Returns the gamepad buttons that are currently pressed. The order of the
    /// buttons in the slice is not guaranteed.
    /// If the core does not support gamepads at the index requested, this should
    /// return `None`.
    fn gamepad_buttons(&self, index: usize) -> Result<Option<&[gamepad::Button]>, Error>;

    /// Returns the menu items that the core supports. This would correspond to the
    /// top level page of config items. If the core does not support a menu, this
    /// should return an empty vector.
    fn menu(&self) -> Result<Vec<CoreMenuItem>, Error>;

    /// Trigger a menu item in the core. This is used to perform an action based on
    /// the menu item selected by the user. It can also be linked to a shortcut.
    fn trigger(&mut self, id: ConfigMenuId) -> Result<(), Error> {
        let _ = id;
        Ok(())
    }

    /// Set an integer option in the core. This is used to set an option that has a
    /// list of choices that can be selected by the user.
    fn int_option(&mut self, id: ConfigMenuId, value: u32) -> Result<(), Error> {
        let _ = id;
        let _ = value;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A core that be used in the Golem platform. This is a wrapper around a core
/// that implements the [`Core`] trait. It can be used to pass around a core
/// without knowing its implementation.
pub struct GolemCore {
    inner: Box<dyn Core + 'static>,
}

impl GolemCore {
    pub fn new(core: impl Core + 'static) -> Self {
        Self {
            inner: Box::new(core),
        }
    }

    pub fn null() -> Self {
        Self::new(NullCore)
    }
}

impl Core for GolemCore {
    fn init(&mut self) -> Result<(), Error> {
        self.inner.init()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn reset(&mut self) -> Result<(), Error> {
        self.inner.reset()
    }

    fn set_volume(&mut self, volume: u8) -> Result<(), Error> {
        self.inner.set_volume(volume)
    }

    fn set_rtc(&mut self, time: SystemTime) -> Result<(), Error> {
        self.inner.set_rtc(time)
    }

    fn screenshot(&self) -> Result<DynamicImage, Error> {
        self.inner.screenshot()
    }

    fn save_state_mut(&mut self, slot: usize) -> Result<Option<&mut dyn SaveState>, Error> {
        self.inner.save_state_mut(slot)
    }

    fn save_state(&self, slot: usize) -> Result<Option<&dyn SaveState>, Error> {
        self.inner.save_state(slot)
    }

    fn mounted_file_mut(&mut self, slot: usize) -> Result<Option<&mut dyn MountedFile>, Error> {
        self.inner.mounted_file_mut(slot)
    }

    fn send_rom(&mut self, rom: Rom) -> Result<(), Error> {
        self.inner.send_rom(rom)
    }

    fn send_bios(&mut self, bios: Bios) -> Result<(), Error> {
        self.inner.send_bios(bios)
    }

    fn key_up(&mut self, key: keyboard::Scancode) -> Result<(), Error> {
        self.inner.key_up(key)
    }

    fn key_down(&mut self, key: keyboard::Scancode) -> Result<(), Error> {
        self.inner.key_down(key)
    }

    fn keys_set(&mut self, keys: &[keyboard::Scancode]) -> Result<(), Error> {
        self.inner.keys_set(keys)
    }

    fn keys(&self) -> Result<&[keyboard::Scancode], Error> {
        self.inner.keys()
    }

    fn gamepad_button_up(&mut self, index: usize, button: gamepad::Button) -> Result<(), Error> {
        self.inner.gamepad_button_up(index, button)
    }

    fn gamepad_button_down(&mut self, index: usize, button: gamepad::Button) -> Result<(), Error> {
        self.inner.gamepad_button_down(index, button)
    }

    fn gamepad_buttons_set(
        &mut self,
        index: usize,
        buttons: &[gamepad::Button],
    ) -> Result<(), Error> {
        self.inner.gamepad_buttons_set(index, buttons)
    }

    fn gamepad_buttons(&self, index: usize) -> Result<Option<&[gamepad::Button]>, Error> {
        self.inner.gamepad_buttons(index)
    }

    fn menu(&self) -> Result<Vec<CoreMenuItem>, Error> {
        self.inner.menu()
    }

    fn trigger(&mut self, id: ConfigMenuId) -> Result<(), Error> {
        self.inner.trigger(id)
    }

    fn int_option(&mut self, id: ConfigMenuId, value: u32) -> Result<(), Error> {
        self.inner.int_option(id, value)
    }

    fn as_any(&self) -> &dyn Any {
        self.inner.as_any()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self.inner.as_any_mut()
    }
}
