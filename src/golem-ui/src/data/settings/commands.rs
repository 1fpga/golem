use crate::input::commands::ShortcutCommand;
use crate::input::shortcut::Shortcut;
use merge::Merge;
use mister_fpga::config_string::ConfigMenu;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;
use tracing::info;

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandSettings {
    #[serde(rename = "perCore")]
    cores: BTreeMap<String, BTreeMap<String, BTreeSet<Shortcut>>>,
    commands: BTreeMap<String, BTreeSet<Shortcut>>,
}

impl Merge for CommandSettings {
    fn merge(&mut self, other: Self) {
        for (k, v) in other.cores {
            let c = self.cores.entry(k).or_default();
            for (k, mut v) in v {
                c.entry(k).or_default().append(&mut v);
            }
        }

        for (k, mut v) in other.commands {
            self.commands.entry(k).or_default().append(&mut v);
        }
    }
}

impl Default for CommandSettings {
    fn default() -> Self {
        let global_shortcuts = ShortcutCommand::globals();

        Self {
            cores: BTreeMap::new(),
            commands: global_shortcuts
                .into_iter()
                .filter_map(|k| {
                    Some((
                        k.setting_name().unwrap().to_string(),
                        BTreeSet::from([k.default_shortcut()?]),
                    ))
                })
                .collect(),
        }
    }
}

impl CommandSettings {
    pub fn all_commands(
        &self,
        core_name: &str,
    ) -> impl Iterator<Item = (ShortcutCommand, Vec<&Shortcut>)> {
        self.cores
            .get(core_name)
            .into_iter()
            .flat_map(|core| {
                core.iter().map(|(cmd, shortcut)| {
                    (
                        ShortcutCommand::CoreSpecificCommand(ConfigMenu::id_from_str(cmd)),
                        shortcut.into_iter().collect::<Vec<_>>(),
                    )
                })
            })
            .chain(self.commands.iter().filter_map(|(cmd, shortcut)| {
                ShortcutCommand::from_str(cmd)
                    .ok()
                    .map(|cmd| (cmd, shortcut.iter().collect::<Vec<_>>()))
            }))
    }

    fn find_core_command_for_id(
        core: &BTreeMap<String, BTreeSet<Shortcut>>,
        id: u32,
    ) -> Option<(&str, &BTreeSet<Shortcut>)> {
        core.iter()
            .find(|(k, _)| ConfigMenu::id_from_str(k) == id)
            .map(|(k, v)| (k.as_str(), v))
    }

    pub fn for_command(
        &self,
        core: Option<&str>,
        command: ShortcutCommand,
    ) -> Option<&BTreeSet<Shortcut>> {
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
            _ => self.commands.get(command.setting_name()?),
        }
    }

    pub fn delete(&mut self, core: Option<&str>, command: ShortcutCommand, shortcut: Shortcut) {
        match command {
            ShortcutCommand::CoreSpecificCommand(id) => {
                if let Some(core) = core {
                    if let Some(core) = self.cores.get_mut(core) {
                        if let Some((key, _)) = Self::find_core_command_for_id(core, id) {
                            let key = key.to_string();
                            if let Some(x) = core.get_mut(&key) {
                                x.retain(|x| *x != shortcut);
                            }
                        }
                    }
                }
            }
            other => {
                if let Some(x) = other.setting_name() {
                    if let Some(x) = self.commands.get_mut(x) {
                        x.retain(|x| *x != shortcut);
                    }
                }
            }
        }
    }

    pub fn clear(&mut self, core: Option<&str>, command: ShortcutCommand) {
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
                    self.commands.remove(x);
                }
            }
        }
    }

    pub fn add_core_specific(&mut self, core: &str, command: &str, shortcut: Shortcut) {
        info!(
            "Setting core-specific command {} for core {} to {:?}",
            command, core, shortcut
        );
        self.cores
            .entry(core.to_string())
            .or_default()
            .entry(command.to_string())
            .or_default()
            .insert(shortcut);
    }

    pub fn add(&mut self, command: ShortcutCommand, shortcut: Shortcut) {
        info!("Adding shortcut `{shortcut:?}` to global command {command}");
        if let Some(name) = command.setting_name() {
            self.commands
                .entry(name.to_string())
                .or_default()
                .insert(shortcut);
        }
    }
}

#[test]
fn serializes() {
    let settings = CommandSettings::default();
    let serialized = json5::to_string(&settings).unwrap();
    assert_eq!(
        serialized,
        r#"{"quit_core":"'F10'","reset_core":"'F11'","show_menu":"'F12'","take_screenshot":"'SysReq'"}"#
    );

    let new_settings = json5::from_str(&serialized).unwrap();
    assert_eq!(settings, new_settings);
}
