use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::input::commands::CoreCommands;
use crate::platform::Core;
use mister_fpga::config_string::ConfigMenu;

mod remap;
use crate::application::GoLEmApp;
use remap::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    Remap(CoreCommands),
    Unselectable,
    Back,
}

impl MenuReturn for MenuAction {
    fn is_selectable(&self) -> bool {
        self != &MenuAction::Unselectable
    }
    fn back() -> Option<Self> {
        Some(MenuAction::Back)
    }
}

pub fn menu(app: &mut GoLEmApp, core: &Option<&mut (impl Core + ?Sized)>) {
    let mut state = None;

    loop {
        let show_menu = app
            .settings()
            .inner()
            .mappings()
            .show_menu
            .as_ref()
            .map(|m| m.to_string());
        let reset_core = app
            .settings()
            .inner()
            .mappings()
            .reset_core
            .as_ref()
            .map(|m| m.to_string());
        let quit_core = app
            .settings()
            .inner()
            .mappings()
            .quit_core
            .as_ref()
            .map(|m| m.to_string());

        let menu = if let Some(c) = core {
            let menu = c.menu_options();
            menu.iter()
                .filter_map(|config_menu| {
                    if let Some(label) = config_menu.label() {
                        let command =
                            CoreCommands::CoreSpecificCommand(ConfigMenu::id_from_str(label));
                        Some((
                            label,
                            app.settings()
                                .inner()
                                .mappings()
                                .for_command(Some(c.name()), command)
                                .map(|x| x.to_string())
                                .unwrap_or_default(),
                            MenuAction::Remap(command),
                        ))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        let items = menu
            .iter()
            .map(|(a, b, c)| (*a, b.as_str(), *c))
            .collect::<Vec<_>>();

        let (result, new_state) = text_menu(
            app,
            "Input Mapping",
            items.as_slice(),
            TextMenuOptions::default().with_state(state).with_prefix(&[
                ("Global Shortcuts (all cores)", "", MenuAction::Unselectable),
                (
                    "Show Menu",
                    if let Some(s) = show_menu.as_ref() {
                        s.as_str()
                    } else {
                        ""
                    },
                    MenuAction::Remap(CoreCommands::ShowCoreMenu),
                ),
                (
                    "Reset Core",
                    if let Some(s) = reset_core.as_ref() {
                        s.as_str()
                    } else {
                        ""
                    },
                    MenuAction::Remap(CoreCommands::QuitCore),
                ),
                (
                    "Quit Core",
                    if let Some(s) = quit_core.as_ref() {
                        s.as_str()
                    } else {
                        ""
                    },
                    MenuAction::Remap(CoreCommands::QuitCore),
                ),
            ]),
        );

        state = Some(new_state);

        match result {
            MenuAction::Unselectable => {}
            MenuAction::Remap(command) => {
                remap(app, core.as_deref(), command);
                continue;
            }

            MenuAction::Back => break,
        }
    }
}
