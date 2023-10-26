use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::application::panels::alert::alert;
use crate::macguiver::application::Application;
use embedded_graphics::pixelcolor::BinaryColor;
use mister_db::models::GameOrder;

#[derive(Default, Debug, Clone, Copy)]
enum MenuAction {
    #[default]
    Back,
    Manage,
    ShowGameInfo(usize),
}

impl MenuReturn for MenuAction {
    fn back() -> Option<Self> {
        Some(MenuAction::Back)
    }
}

fn build_games_list_(database: &mut mister_db::Connection) -> Vec<mister_db::models::Game> {
    let all_games = mister_db::models::Game::list(database, 0, 1000, GameOrder::LastPlayed);
    match all_games {
        Ok(all_games) => all_games,
        Err(e) => {
            tracing::error!("Database error: {e}");
            Vec::new()
        }
    }
}

pub fn games_list(app: &mut impl Application<Color = BinaryColor>) {
    let mut state = None;
    let all_games = build_games_list_(&mut app.database().lock().unwrap());

    let menu_items = all_games
        .iter()
        .enumerate()
        .map(|(i, game)| (game.name.as_str(), "", MenuAction::ShowGameInfo(i)))
        .collect::<Vec<_>>();

    loop {
        let (result, new_state) = text_menu(
            app,
            "Games",
            &menu_items,
            TextMenuOptions::default().with_state(state).with_suffix(&[(
                "Manage Games",
                "",
                MenuAction::Manage,
            )]),
        );
        state = Some(new_state);

        match result {
            MenuAction::Back => break,
            MenuAction::Manage => {
                alert(
                    app,
                    "Not implemented yet",
                    "This feature is not implemented yet",
                    &["Okay"],
                );
            }
            MenuAction::ShowGameInfo(_) => {
                alert(
                    app,
                    "Not implemented yet",
                    "This feature is not implemented yet",
                    &["Okay"],
                );
            }
        }
    }
}
