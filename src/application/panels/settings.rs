use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::application::panels::alert::alert;
use crate::macguiver::application::Application;
use crate::platform::Core;
use embedded_graphics::pixelcolor::BinaryColor;

#[derive(Debug, Clone, Copy, PartialEq)]
enum MenuAction {
    ChangeTimestampFormat,
    InputMapping,
    ResetAll,
    Back,
}

impl MenuReturn for MenuAction {
    fn back() -> Option<Self> {
        Some(Self::Back)
    }
}

pub fn settings_panel(
    app: &mut impl Application<Color = BinaryColor>,
    core: &Option<&mut (impl Core + ?Sized)>,
) {
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
