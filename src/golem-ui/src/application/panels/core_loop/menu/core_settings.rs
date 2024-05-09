use std::convert::TryFrom;

use tracing::info;

use golem_core::runner::CoreLaunchInfo;
use golem_core::{Core, GolemCore};
use mister_fpga::config_string::ConfigMenu;
use mister_fpga::core::MisterFpgaCore;
use mister_fpga::types::StatusBitMap;

use crate::application::coordinator::GameStartInfo;
use crate::application::menu::filesystem::{select_file_path_menu, FilesystemMenuOptions};
use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuItem, TextMenuOptions};
use crate::application::GoLEmApp;
use crate::data::paths::core_root_path;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum CoreMenuAction {
    LoadFile(u8),
    Trigger(u8, bool),
    ToggleOption(u8, u8, usize, usize),
    #[default]
    Back,
}

impl MenuReturn for CoreMenuAction {
    fn back() -> Option<Self> {
        Some(Self::Back)
    }
}

pub fn into_text_menu_item<'a>(
    item: &'a ConfigMenu,
    status: &StatusBitMap,
) -> Option<TextMenuItem<'a, CoreMenuAction>> {
    match item {
        ConfigMenu::Empty(None) => Some(TextMenuItem::separator()),
        ConfigMenu::Empty(Some(title)) => Some(TextMenuItem::unselectable(title)),
        ConfigMenu::Option {
            bits,
            label,
            choices,
        } => {
            let from = bits.start;
            let to = bits.end;
            let value = status.get_range(from..to);
            let value = usize::try_from(value).unwrap_or_default();
            Some(TextMenuItem::navigation_item(
                label,
                &choices[value],
                CoreMenuAction::ToggleOption(from, to, value, choices.len()),
            ))
        }
        ConfigMenu::Trigger {
            close_osd,
            index,
            label,
        } => Some(TextMenuItem::navigation_item(
            label,
            if *close_osd { "<-" } else { "" },
            CoreMenuAction::Trigger(*index, *close_osd),
        )),
        ConfigMenu::HideIf(b, sub) => {
            if status.get(*b as usize) {
                None
            } else {
                into_text_menu_item(sub, status)
            }
        }
        ConfigMenu::HideUnless(b, sub) => {
            if status.get(*b as usize) {
                into_text_menu_item(sub, status)
            } else {
                None
            }
        }
        ConfigMenu::DisableIf(b, sub) => {
            if status.get(*b as usize) {
                into_text_menu_item(sub, status)
            } else if let Some(item) = into_text_menu_item(sub, status) {
                Some(item.disabled())
            } else {
                None
            }
        }
        ConfigMenu::DisableUnless(b, sub) => {
            if status.get(*b as usize) {
                if let Some(item) = into_text_menu_item(sub, status) {
                    Some(item.disabled())
                } else {
                    None
                }
            } else {
                into_text_menu_item(sub, status)
            }
        }
        ConfigMenu::LoadFile(info) => {
            const DEFAULT_LABEL: &str = "Load File";
            Some(TextMenuItem::navigation_item(
                info.label.as_deref().unwrap_or(DEFAULT_LABEL),
                info.marker.as_str(),
                CoreMenuAction::LoadFile(info.index),
            ))
        }
        ConfigMenu::LoadFileAndRemember(info) => {
            const DEFAULT_LABEL: &str = "Load File";
            Some(TextMenuItem::navigation_item(
                info.label.as_deref().unwrap_or(DEFAULT_LABEL),
                info.marker.as_str(),
                CoreMenuAction::LoadFile(info.index),
            ))
        }
        ConfigMenu::PageItem(_index, sub) => {
            // TODO: add full page support.
            into_text_menu_item(sub, status)
        }
        _ => None,
    }
}

pub fn execute_core_settings(
    app: &mut GoLEmApp,
    core: &mut MisterFpgaCore,
    action: CoreMenuAction,
) -> Option<bool> {
    match action {
        CoreMenuAction::Back => {
            return Some(false);
        }
        CoreMenuAction::ToggleOption(from, to, value, max) => {
            let mut bits = *core.status_bits();
            bits.set_range(from..to, ((value + 1) % max) as u32);
            core.send_status_bits(bits);
        }
        CoreMenuAction::Trigger(idx, close_osd) => {
            core.status_pulse(idx as usize);
            if close_osd {
                return Some(true);
            }
        }
        CoreMenuAction::LoadFile(index) => {
            let maybe_info = core
                .menu_options()
                .iter()
                .filter_map(|c| match c {
                    ConfigMenu::LoadFile(info) if info.index == index => Some(info),
                    ConfigMenu::LoadFileAndRemember(info) if info.index == index => Some(info),
                    _ => None,
                })
                .next();
            if maybe_info.is_none() {
                return None;
            }
            let info = maybe_info.unwrap().as_ref().clone();

            let path = select_file_path_menu(
                app,
                "Select File",
                core_root_path(),
                FilesystemMenuOptions::default()
                    .with_allow_back(true)
                    .with_extensions(info.extensions.iter().map(|x| x.to_string()).collect()),
            )
            .unwrap();

            let p = match path {
                None => return None,
                Some(p) => p,
            };
            info!("Loading file {:?}", p);
            let index = info.index;
            let mut should_load = true;
            if index == 0 {
                let maybe_id = golem_db::models::Game::get_by_path(
                    &mut app.database.lock().unwrap(),
                    &p.to_string_lossy(),
                )
                .map_err(|e| e.to_string())
                .unwrap_or(None)
                .map(|g| g.id);
                if let Some(game_id) = maybe_id {
                    app.coordinator_mut()
                        .launch_game(
                            app,
                            CoreLaunchInfo::current()
                                .with_data(GameStartInfo::default().with_game_id(game_id)),
                        )
                        .unwrap();
                    should_load = false;
                }
            }

            if should_load {
                core.load_file(&p, Some(info)).unwrap();
            }

            return Some(true);
        }
    }

    None
}

/// The Core Settings menu. We cannot use `text_menu` here as we need to generate
/// custom menu lines for some items.
/// Returns whether we should close the OSD or not.
pub fn core_settings(app: &mut GoLEmApp, core: &mut MisterFpgaCore) -> bool {
    let mut state = None;
    loop {
        let status = *core.status_bits();
        let mut items = core
            .menu_options()
            .iter()
            .filter_map(|i| into_text_menu_item(i, &status))
            .collect::<Vec<_>>();

        let (action, new_state) = text_menu(
            app,
            "Core Settings",
            &mut items,
            TextMenuOptions::default().with_state(state),
        );
        state = Some(new_state);

        if let Some(x) = execute_core_settings(app, core, action) {
            break x;
        }
    }
}
