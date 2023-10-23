use crate::application::menu::cores_menu_panel;
use crate::application::menu::style::MenuReturn;
use crate::application::menu::text_menu;
use crate::application::panels::alert::alert;
use crate::application::panels::input_tester::input_tester;
use crate::application::panels::settings::settings_panel;
use crate::macguiver::application::Application;
use embedded_graphics::pixelcolor::BinaryColor;
use mister_db::diesel::{QueryDsl, RunQueryDsl};
use mister_db::schema::cores::dsl::cores;
use std::ops::DerefMut;

#[derive(Clone, Copy, Debug, PartialEq)]
enum MenuAction {
    Games,
    Cores,
    Settings,
    InputTester,
    About,
    Idle,
}

impl MenuReturn for MenuAction {
    fn back() -> Self {
        MenuAction::Idle
    }
}

pub fn main_menu(app: &mut impl Application<Color = BinaryColor>) {
    let ncores: i64 = cores
        .count()
        .get_result(app.database().write().unwrap().deref_mut())
        .unwrap();
    let mut state = None;

    loop {
        let (result, new_state) = text_menu(
            app,
            " ",
            &[
                ("Cores", &format!("({ncores})"), MenuAction::Cores),
                ("Games", "(0)", MenuAction::Games),
                ("Settings...", "", MenuAction::Settings),
                ("Input Tester", "", MenuAction::InputTester),
                ("About", "", MenuAction::About),
            ],
            state,
        );
        state = Some(new_state);

        match result {
            MenuAction::Games => {
                alert(app, "Games", "Not implemented", &["Okay"]);
            }
            MenuAction::Cores => cores_menu_panel(app),
            MenuAction::Settings => settings_panel(app),
            MenuAction::InputTester => input_tester(app),
            MenuAction::About => {
                alert(app, "About", "Not implemented", &["Okay"]);
            }
            _ => {}
        }
    }
}
