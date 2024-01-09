use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::input::commands::ShortcutCommand;
use crate::platform::Core;
use mister_fpga::config_string::ConfigMenu;

mod remap;
use crate::application::GoLEmApp;
use remap::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    Remap(ShortcutCommand),
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

pub fn menu<C: Core + ?Sized>(app: &mut GoLEmApp, core: &Option<&mut C>) {
    let mut state = None;

    loop {
        let global_shortcuts = ShortcutCommand::globals()
            .into_iter()
            .filter_map(|command| {
                Some((
                    command.setting_name()?,
                    app.settings()
                        .inner()
                        .mappings()
                        .for_command(None, command)
                        .map(|x| {
                            if x.len() > 1 {
                                "Multiple".to_string()
                            } else {
                                let result = x.first().unwrap().to_string();
                                if result.len() > 30 {
                                    format!("{}...", &result[..30])
                                } else {
                                    result
                                }
                            }
                        })
                        .unwrap_or_default(),
                    MenuAction::Remap(command),
                ))
            })
            .collect::<Vec<_>>();
        let global_items = global_shortcuts
            .iter()
            .map(|(a, b, c)| (*a, b.as_str(), *c))
            .collect::<Vec<_>>();

        let menu = if let Some(c) = &core {
            let menu = c.menu_options();
            menu.iter()
                .filter_map(|config_menu| {
                    if let Some(label) = config_menu.label() {
                        let command =
                            ShortcutCommand::CoreSpecificCommand(ConfigMenu::id_from_str(label));
                        Some((
                            label,
                            app.settings()
                                .inner()
                                .mappings()
                                .for_command(Some(c.name()), command)
                                .map(|x| {
                                    x.iter()
                                        .map(|x| x.to_string())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                })
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
            TextMenuOptions::default().with_state(state).with_prefix(
                [
                    vec![("Global Shortcuts (all cores)", "", MenuAction::Unselectable)],
                    global_items,
                ]
                .concat()
                .as_slice(),
            ),
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
