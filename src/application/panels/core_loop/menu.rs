use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::macguiver::application::Application;
use crate::platform::{Core, CoreManager, GoLEmPlatform};
use embedded_graphics::pixelcolor::BinaryColor;

mod core_debug;
mod core_settings;
mod items;

#[derive(Copy, Clone, PartialEq)]
enum CoreMenuAction {
    Reset,
    CoreSettings,
    DebugInfo,
    Back,
    Quit,

    Unselectable,
}

impl MenuReturn for CoreMenuAction {
    fn is_selectable(&self) -> bool {
        *self != Self::Unselectable
    }

    fn back() -> Option<Self> {
        Some(Self::Back)
    }
}

/// Shows the core menu and interact with it.
/// This will return `true` if the user decided to quit the core (in which case the
/// main MENU should be reloaded).
pub fn core_menu(app: &mut impl Application<Color = BinaryColor>, core: &mut impl Core) -> bool {
    app.platform_mut().core_manager_mut().show_menu();
    let mut state = None;

    let result = loop {
        let version = core
            .version()
            .map(|s| ("Version", s, CoreMenuAction::Unselectable));
        let (result, new_state) = text_menu(
            app,
            "Core",
            &[
                ("Reset Core", "", CoreMenuAction::Reset),
                ("Core Settings", "", CoreMenuAction::CoreSettings),
                ("Debug Info", "", CoreMenuAction::DebugInfo),
            ],
            TextMenuOptions::default()
                .with_state(state)
                .with_suffix(&[
                    version.unwrap_or(("", "", CoreMenuAction::Unselectable)),
                    ("Back", "<-", CoreMenuAction::Back),
                    ("Quit Core", "", CoreMenuAction::Quit),
                ])
                .with_back_menu(false),
        );
        state = Some(new_state);

        match result {
            CoreMenuAction::Reset => {
                core.status_pulse(0);
            }
            CoreMenuAction::CoreSettings => {
                if core_settings::core_settings(app, core) {
                    break false;
                }
            }
            CoreMenuAction::DebugInfo => {
                core_debug::debug_info(app, core);
            }
            CoreMenuAction::Back => {
                break false;
            }
            CoreMenuAction::Quit => {
                break true;
            }

            CoreMenuAction::Unselectable => unreachable!(),
        }
    };

    app.platform_mut().core_manager_mut().hide_menu();
    result
}
