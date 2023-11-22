use crate::input::commands::CoreCommands;
use crate::input::BasicInputShortcut;
use sdl3::keyboard::Scancode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct MappingSettings {
    #[serde(default)]
    pub show_menu: Option<BasicInputShortcut>,
    #[serde(default)]
    pub quit_core: Option<BasicInputShortcut>,
    // #[serde(default)]
    // pub save_state: Option<()>,
    // #[serde(default)]
    // pub load_save_state: Option<()>,
}

impl Default for MappingSettings {
    fn default() -> Self {
        Self {
            show_menu: Some(BasicInputShortcut::default().with_key(Scancode::F12)),
            quit_core: None,
        }
    }
}

impl MappingSettings {
    pub fn for_command(&self, command: CoreCommands) -> Option<&BasicInputShortcut> {
        match command {
            CoreCommands::ShowCoreMenu => self.show_menu.as_ref(),
            CoreCommands::QuitCore => self.quit_core.as_ref(),
        }
    }

    pub fn delete(&mut self, command: CoreCommands) {
        match command {
            CoreCommands::ShowCoreMenu => self.show_menu = None,
            CoreCommands::QuitCore => self.quit_core = None,
        }
    }

    pub fn set(&mut self, command: CoreCommands, shortcut: BasicInputShortcut) {
        match command {
            CoreCommands::ShowCoreMenu => self.show_menu = Some(shortcut),
            CoreCommands::QuitCore => self.quit_core = Some(shortcut),
        }
    }
}
