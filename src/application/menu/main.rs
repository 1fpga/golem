use crate::application::menu::games::games_list;
use crate::application::menu::style::MenuReturn;
use crate::application::menu::text_menu;
use crate::application::menu::{cores_menu_panel, TextMenuOptions};
use crate::application::panels::alert::alert;
use crate::application::panels::settings::settings_panel;
use crate::application::panels::tools::tools_menu;
use crate::macguiver::application::Application;
use embedded_graphics::pixelcolor::BinaryColor;
use golem_db::models;

#[derive(Default, Clone, Copy, Debug, PartialEq)]
enum MenuAction {
    Games,
    Cores,
    Settings,
    About,
    Tools,
    #[default]
    Idle,
}

impl MenuReturn for MenuAction {}

pub fn main_menu(app: &mut impl Application<Color = BinaryColor>) {
    let mut state = None;

    loop {
        let (ncores, ngames) = {
            let db = app.database();
            let mut db = db.lock().unwrap();
            let ncores: i64 = models::Core::count(&mut db).unwrap();
            let ngames = models::Game::count(&mut db).unwrap();
            (ncores, ngames)
        };
        let (result, new_state) = text_menu(
            app,
            " ",
            &[
                ("Cores", format!("({ncores})"), MenuAction::Cores),
                ("Games", format!("({ngames})"), MenuAction::Games),
                ("Settings...", "".to_string(), MenuAction::Settings),
                ("Tools...", "".to_string(), MenuAction::Tools),
                ("About", "".to_string(), MenuAction::About),
            ],
            TextMenuOptions::default().with_state(state),
        );
        state = Some(new_state);

        match result {
            MenuAction::Games => games_list(app),
            MenuAction::Cores => cores_menu_panel(app),
            MenuAction::Settings => settings_panel(app),
            MenuAction::Tools => tools_menu(app),
            MenuAction::About => {
                alert(app, "About", "Not implemented", &["Okay"]);
            }
            _ => {}
        }
    }
}
