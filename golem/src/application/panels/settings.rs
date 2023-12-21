use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::application::panels::alert::alert;
use crate::application::GoLEmApp;
use crate::platform::Core;

#[derive(Debug, Clone, Copy, PartialEq)]
enum MenuAction {
    ChangeTimestampFormat,
    ShowFps,
    InvertToolbar,
    InputMapping,
    ResetAll,
    Back,
}

impl MenuReturn for MenuAction {
    fn back() -> Option<Self> {
        Some(Self::Back)
    }
}

pub fn settings_panel(app: &mut GoLEmApp, core: &Option<&mut (impl Core + ?Sized)>) {
    let mut state = None;
    loop {
        let (result, new_state) = text_menu(
            app,
            "Settings",
            &[
                (
                    "Change timestamp format",
                    app.settings()
                        .toolbar_datetime_format()
                        .to_string()
                        .as_str(),
                    MenuAction::ChangeTimestampFormat,
                ),
                (
                    "Show FPS",
                    if app.settings().show_fps() {
                        "On"
                    } else {
                        "Off"
                    },
                    MenuAction::ShowFps,
                ),
                (
                    "Invert Header Bar",
                    if app.settings().invert_toolbar() {
                        "On"
                    } else {
                        "Off"
                    },
                    MenuAction::InvertToolbar,
                ),
                ("Input Mapping", "", MenuAction::InputMapping),
                ("Reset all settings", "", MenuAction::ResetAll),
            ],
            TextMenuOptions::default().with_state(state),
        );

        state = Some(new_state);

        match result {
            MenuAction::ChangeTimestampFormat => {
                app.settings().toggle_toolbar_datetime_format();
            }
            MenuAction::ShowFps => {
                app.settings().toggle_show_fps();
            }
            MenuAction::InvertToolbar => {
                app.settings().toggle_invert_toolbar();
            }
            MenuAction::InputMapping => {
                crate::application::panels::core_loop::menu::input_mapping::menu(app, core);
            }
            MenuAction::ResetAll => {
                let ok = alert(app, "Reset all settings?", "This will reset all settings and downloaded files to their default values. Are you sure?", &["Yes", "No"]);
                if ok == Some(0) {
                    app.settings().reset_all_settings();
                }
            }
            MenuAction::Back => break,
        }
    }
}
