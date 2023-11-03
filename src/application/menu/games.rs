use crate::application::menu::games::manage::manage_games;
use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::application::panels::alert::alert;
use crate::macguiver::application::Application;
use embedded_graphics::pixelcolor::BinaryColor;
use mister_db::models::GameOrder;

mod manage;

#[derive(Default, Debug, Clone, Copy)]
enum MenuAction {
    #[default]
    Back,
    ChangeSort,
    Manage,
    LoadGame(usize),
    ShowDetails(usize),
}

impl MenuReturn for MenuAction {
    fn back() -> Option<Self> {
        Some(MenuAction::Back)
    }
    fn sort() -> Option<Self> {
        Some(Self::ChangeSort)
    }
    fn into_details(self) -> Option<Self> {
        match self {
            MenuAction::LoadGame(i) => Some(MenuAction::ShowDetails(i)),
            _ => None,
        }
    }
}

fn build_games_list_(
    database: &mut mister_db::Connection,
    sort: GameOrder,
) -> Vec<mister_db::models::Game> {
    let all_games = mister_db::models::Game::list(database, 0, 1000, sort);
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
    let mut sort_order = GameOrder::LastPlayed;

    loop {
        let all_games = build_games_list_(&mut app.database().lock().unwrap(), sort_order);

        let menu_items = all_games
            .iter()
            .enumerate()
            .map(|(i, game)| (game.name.as_str(), "", MenuAction::LoadGame(i)))
            .collect::<Vec<_>>();

        let (result, new_state) = text_menu(
            app,
            "Games",
            &menu_items,
            TextMenuOptions::default()
                .with_state(state)
                .with_sort(sort_order.as_str())
                .with_suffix(&[("Manage Games", "", MenuAction::Manage)])
                .with_details("Details"),
        );
        state = Some(new_state);

        match result {
            MenuAction::Back => break,
            MenuAction::Manage => manage_games(app),
            MenuAction::LoadGame(_) => {
                alert(
                    app,
                    "Not implemented yet",
                    "This feature is not implemented yet",
                    &["Okay"],
                );
            }
            MenuAction::ShowDetails(_) => {
                alert(
                    app,
                    "Not implemented yet",
                    "This feature is not implemented yet",
                    &["Okay"],
                );
            }
            MenuAction::ChangeSort => sort_order = sort_order.next(),
        }
    }
}
