use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::input::commands::CoreCommands;
use crate::macguiver::application::Application;
use crate::platform::Core;
use embedded_graphics::pixelcolor::BinaryColor;

mod remap;
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

pub fn menu<C: Core>(app: &mut impl Application<Color = BinaryColor>, _core: Option<&mut C>) {
    let mut state = None;

    loop {
        let show_menu = app
            .settings()
            .inner()
            .mappings()
            .show_menu
            .as_ref()
            .map(|m| m.to_string());
        let quit_core = app
            .settings()
            .inner()
            .mappings()
            .quit_core
            .as_ref()
            .map(|m| m.to_string());

        let (result, new_state) = text_menu(
            app,
            "Input Mapping",
            &[
                ("Global Shortcuts", "", MenuAction::Unselectable),
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
                    "Quit Core",
                    if let Some(s) = quit_core.as_ref() {
                        s.as_str()
                    } else {
                        ""
                    },
                    MenuAction::Remap(CoreCommands::QuitCore),
                ),
            ],
            TextMenuOptions::default().with_state(state),
        );

        state = Some(new_state);

        match result {
            MenuAction::Unselectable => {}
            MenuAction::Remap(command) => {
                remap(app, command);
                continue;
            }

            MenuAction::Back => break,
        }
    }
}
