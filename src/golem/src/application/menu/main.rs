use crate::application::menu::games::games_list;
use crate::application::menu::style::MenuReturn;
use crate::application::menu::text_menu;
use crate::application::menu::{cores_menu_panel, TextMenuOptions};
use crate::application::panels::alert::alert;
use crate::application::panels::settings::settings_panel;
use crate::application::panels::tools::tools_menu;
use crate::application::GoLEmApp;
use golem_db::models;

#[derive(Default, Clone, Copy, Debug, PartialEq)]
enum MenuAction {
    Games,
    Cores,
    Settings,
    About,
    Tools,
    Quit,
    #[default]
    Idle,
}

impl MenuReturn for MenuAction {}

pub fn main_menu(app: &mut GoLEmApp) {
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
                ("Games", format!("({ngames})").as_str(), MenuAction::Games),
                ("Cores", format!("({ncores})").as_str(), MenuAction::Cores),
                ("Settings...", "", MenuAction::Settings),
                ("Tools...", "", MenuAction::Tools),
                ("About", "", MenuAction::About),
                ("-", "", MenuAction::Idle),
                ("Quit GoLEm (and go back to MiSTer)", "", MenuAction::Quit),
            ],
            TextMenuOptions::default().with_state(state),
        );
        state = Some(new_state);

        match result {
            MenuAction::Games => games_list(app),
            MenuAction::Cores => cores_menu_panel(app),
            MenuAction::Settings => settings_panel(app, &None),
            MenuAction::Tools => tools_menu(app),
            MenuAction::About => {
                alert(app, "About", "Not implemented", &["Okay"]);
            }
            MenuAction::Quit => {
                return;
            }
            MenuAction::Idle => {}
        }
    }
}
