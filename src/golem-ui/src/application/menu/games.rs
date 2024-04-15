use anyhow::anyhow;
use thiserror::__private::AsDynError;

use golem_db::models::GameOrder;

use crate::application::coordinator::GameStartInfo;
use crate::application::GoLEmApp;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::application::menu::games::manage::manage_games;
use crate::application::menu::style::MenuReturn;
use crate::application::panels::alert::show_error;
use crate::application::panels::core_loop::run_core_loop;

mod details;
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
    fn into_details(self) -> Option<Self> {
        match self {
            MenuAction::LoadGame(i) => Some(MenuAction::ShowDetails(i)),
            _ => None,
        }
    }
    fn sort() -> Option<Self> {
        Some(Self::ChangeSort)
    }
}

fn build_games_list_(
    database: &mut golem_db::Connection,
    sort: GameOrder,
) -> Vec<golem_db::models::Game> {
    let all_games = golem_db::models::Game::list(database, 0, 1000, sort);
    all_games.unwrap_or_else(|e| {
        tracing::error!("Database error: {e}");
        Vec::new()
    })
}
