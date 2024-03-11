use crate::application::menu::filesystem::{select_file_path_menu, FilesystemMenuOptions};
use crate::application::menu::style::MenuReturn;
use crate::application::menu::text_menu;
use crate::application::menu::TextMenuOptions;
use crate::application::panels::alert::{alert, show_error};
use crate::application::panels::input_tester::input_tester;
use crate::application::GoLEmApp;
use anyhow::anyhow;
use thiserror::__private::AsDynError;

mod menu_tester;
mod progress_tester;

#[derive(Copy, Clone, Debug, PartialEq)]
enum ToolMenuReturn {
    InputTester,
    MenuTester,
    MenuTesterBinNes,
    ProgressTester,
    ErrorTester,
    Back,
}

impl MenuReturn for ToolMenuReturn {
    fn back() -> Option<Self> {
        Some(ToolMenuReturn::Back)
    }
}

pub fn tools_menu(app: &mut GoLEmApp) {
    let mut state = None;

    loop {
        let (result, new_state) = text_menu(
            app,
            "Tools",
            &[
                ("Input Tester", "", ToolMenuReturn::InputTester),
                ("Menu Tester", "", ToolMenuReturn::MenuTester),
                (
                    "Menu Tester (bin, nes)",
                    "",
                    ToolMenuReturn::MenuTesterBinNes,
                ),
                ("Progress Tester", "", ToolMenuReturn::ProgressTester),
                ("Error Tester", "", ToolMenuReturn::ErrorTester),
            ],
            TextMenuOptions::default().with_state(state),
        );
        state = Some(new_state);

        match result {
            ToolMenuReturn::InputTester => input_tester(app),
            ToolMenuReturn::MenuTester => menu_tester::menu_tester(app),
            ToolMenuReturn::ProgressTester => progress_tester::progress_tester(app),
            ToolMenuReturn::MenuTesterBinNes => {
                let p = select_file_path_menu(
                    app,
                    "Select a file",
                    dirs::desktop_dir().unwrap_or(std::env::current_dir().unwrap()),
                    FilesystemMenuOptions::default()
                        .with_allow_back(true)
                        .with_extensions(vec!["bin".to_string(), "nes".to_string()]),
                );
                alert(app, "Selected", &format!("Selected: {:?}", p), &["Okay"]);
            }
            ToolMenuReturn::ErrorTester => show_error(
                app,
                anyhow!("Test Error. This is a test error. Please do not report this.")
                    .as_dyn_error(),
                true,
            ),
            ToolMenuReturn::Back => break,
        }
    }
}
