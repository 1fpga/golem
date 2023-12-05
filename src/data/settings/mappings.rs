use crate::input::commands::ShortcutCommand;
use crate::input::Shortcut;
use mister_fpga::config_string::ConfigMenu;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;
use tracing::info;

#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq)]
pub struct MappingSettings {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    cores: BTreeMap<String, BTreeMap<String, Shortcut>>,

    #[serde(flatten)]
    shortcuts: BTreeMap<String, Shortcut>,
}

impl Default for MappingSettings {
    fn default() -> Self {
        let global_shortcuts = ShortcutCommand::globals();

        Self {
            cores: BTreeMap::new(),
            shortcuts: global_shortcuts
                .into_iter()
                .filter_map(|k| {
                    Some((k.setting_name().unwrap().to_string(), k.default_shortcut()?))
                })
                .collect(),
        }
    }
}

impl MappingSettings {
    pub fn all_commands(
        &self,
        core_name: &str,
    ) -> impl Iterator<Item = (ShortcutCommand, &Shortcut)> {
        self.cores
            .get(core_name)
            .into_iter()
            .flat_map(|core| {
                core.iter().map(|(cmd, shortcut)| {
                    (
                        ShortcutCommand::CoreSpecificCommand(ConfigMenu::id_from_str(cmd)),
                        shortcut,
                    )
                })
            })
            .chain(self.shortcuts.iter().filter_map(|(cmd, shortcut)| {
                ShortcutCommand::from_str(cmd)
                    .ok()
                    .map(|cmd| (cmd, shortcut))
            }))
    }

    fn find_core_command_for_id(
        core: &BTreeMap<String, Shortcut>,
        id: u32,
    ) -> Option<(&str, &Shortcut)> {
        core.iter()
            .find(|(k, _)| ConfigMenu::id_from_str(k) == id)
            .map(|(k, v)| (k.as_str(), v))
    }

    pub fn for_command(&self, core: Option<&str>, command: ShortcutCommand) -> Option<&Shortcut> {
        match command {
            ShortcutCommand::CoreSpecificCommand(id) => {
                if let Some(core) = core {
                    self.cores
                        .get(core)
                        .and_then(|core| Self::find_core_command_for_id(core, id).map(|(_, v)| v))
                } else {
                    None
                }
            }
            _ => self.shortcuts.get(command.setting_name()?),
        }
    }

    pub fn delete(&mut self, core: Option<&str>, command: ShortcutCommand) {
        match command {
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
            other => {
                if let Some(x) = other.setting_name() {
                    self.shortcuts.remove(x);
                }
            }
        }
    }

    pub fn set_core_specific(&mut self, core: &str, command: &str, shortcut: Shortcut) {
        info!(
            "Setting core-specific command {} for core {} to {:?}",
            command, core, shortcut
        );
        self.cores
            .entry(core.to_string())
            .or_default()
            .insert(command.to_string(), shortcut);
    }

    pub fn set(&mut self, command: ShortcutCommand, shortcut: Shortcut) {
        info!("Setting global command {} to {:?}", command, shortcut);
        if let Some(name) = command.setting_name() {
            self.shortcuts.insert(name.to_string(), shortcut);
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
