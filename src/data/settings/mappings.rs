use crate::input::commands::ShortcutCommand;
use crate::input::BasicInputShortcut;
use mister_fpga::config_string::ConfigMenu;
use sdl3::keyboard::Scancode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tracing::info;

#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq)]
pub struct MappingSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_menu: Option<BasicInputShortcut>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_core: Option<BasicInputShortcut>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quit_core: Option<BasicInputShortcut>,
    // #[serde(default)]
    // pub save_state: Option<()>,
    // #[serde(default)]
    // pub load_save_state: Option<()>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    cores: BTreeMap<String, BTreeMap<String, BasicInputShortcut>>,
}

impl Default for MappingSettings {
    fn default() -> Self {
        Self {
            show_menu: Some(BasicInputShortcut::default().with_key(Scancode::F12)),
            reset_core: None,
            quit_core: None,
            cores: BTreeMap::new(),
        }
    }
}

impl MappingSettings {
    pub fn core_commands(
        &self,
        core_name: &str,
    ) -> impl Iterator<Item = (ShortcutCommand, &BasicInputShortcut)> {
        self.cores.get(core_name).into_iter().flat_map(|core| {
            core.iter().map(|(cmd, shortcut)| {
                (
                    ShortcutCommand::CoreSpecificCommand(ConfigMenu::id_from_str(cmd)),
                    shortcut,
                )
            })
        })
    }

    pub fn global_commands(&self) -> impl Iterator<Item = (ShortcutCommand, &BasicInputShortcut)> {
        vec![
            (ShortcutCommand::ShowCoreMenu, self.show_menu.as_ref()),
            (ShortcutCommand::ResetCore, self.reset_core.as_ref()),
            (ShortcutCommand::QuitCore, self.quit_core.as_ref()),
        ]
        .into_iter()
        .filter_map(|(cmd, shortcut)| shortcut.map(|s| (cmd, s)))
    }

    fn find_core_command_for_id(
        core: &BTreeMap<String, BasicInputShortcut>,
        id: u32,
    ) -> Option<(&str, &BasicInputShortcut)> {
        core.iter()
            .find(|(k, _)| ConfigMenu::id_from_str(k) == id)
            .map(|(k, v)| (k.as_str(), v))
    }

    pub fn for_command(
        &self,
        core: Option<&str>,
        command: ShortcutCommand,
    ) -> Option<&BasicInputShortcut> {
        match command {
            ShortcutCommand::ShowCoreMenu => self.show_menu.as_ref(),
            ShortcutCommand::ResetCore => self.reset_core.as_ref(),
            ShortcutCommand::QuitCore => self.quit_core.as_ref(),
            ShortcutCommand::CoreSpecificCommand(id) => {
                if let Some(core) = core {
                    self.cores
                        .get(core)
                        .and_then(|core| Self::find_core_command_for_id(core, id).map(|(_, v)| v))
                } else {
                    None
                }
            }
        }
    }

    pub fn delete(&mut self, core: Option<&str>, command: ShortcutCommand) {
        match command {
            ShortcutCommand::ShowCoreMenu => self.show_menu = None,
            ShortcutCommand::ResetCore => self.reset_core = None,
            ShortcutCommand::QuitCore => self.quit_core = None,
            ShortcutCommand::CoreSpecificCommand(id) => {
                if let Some(core) = core {
                    if let Some(core) = self.cores.get_mut(core) {
                        if let Some((key, _)) = Self::find_core_command_for_id(core, id) {
                            let key = key.to_string();
                            core.remove(&key);
                        }
                    }
                }
            }
        }
    }

    pub fn set_core_specific(&mut self, core: &str, command: &str, shortcut: BasicInputShortcut) {
        info!(
            "Setting core-specific command {} for core {} to {:?}",
            command, core, shortcut
        );
        self.cores
            .entry(core.to_string())
            .or_default()
            .insert(command.to_string(), shortcut);
    }

    pub fn set(&mut self, command: ShortcutCommand, shortcut: BasicInputShortcut) {
        match command {
            ShortcutCommand::ShowCoreMenu => self.show_menu = Some(shortcut),
            ShortcutCommand::ResetCore => self.reset_core = Some(shortcut),
            ShortcutCommand::QuitCore => self.quit_core = Some(shortcut),
            _ => {}
        }
    }
}

#[test]
fn serializes() {
    let settings = MappingSettings::default();
    let serialized = json5::to_string(&settings).unwrap();
    assert_eq!(serialized, r#"{"show_menu":{"keys":["F12"]}}"#);

    let new_settings = json5::from_str(&serialized).unwrap();
    assert_eq!(settings, new_settings);
}
