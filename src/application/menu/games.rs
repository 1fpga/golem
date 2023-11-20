use crate::application::menu::games::manage::manage_games;
use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::application::panels::alert::show_error;
use crate::application::panels::core_loop::run_core_loop;
use crate::macguiver::application::Application;
use crate::platform::{Core, CoreManager, GoLEmPlatform};
use anyhow::anyhow;
use embedded_graphics::pixelcolor::BinaryColor;
use golem_db::models::GameOrder;
use std::path::PathBuf;
use thiserror::__private::AsDynError;

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
        let mut all_games = build_games_list_(&mut app.database().lock().unwrap(), sort_order);

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
                .with_suffix(&[("Manage Games", "", MenuAction::Manage).into()])
                .with_details("Details"),
        );
        state = Some(new_state);

        match result {
            MenuAction::Back => break,
            MenuAction::Manage => manage_games(app),
            MenuAction::LoadGame(i) => {
                let game = &mut all_games[i];
                let file_path = match game.path.as_ref() {
                    Some(path) => path,
                    None => {
                        show_error(app, anyhow!("No file path for game").as_dyn_error(), true);
                        return;
                    }
                };
                let file_path = PathBuf::from(file_path);
                let core_path = if let Some(core_id) = game.core_id {
                    let core =
                        golem_db::models::Core::get(&mut app.database().lock().unwrap(), core_id)
                            .unwrap()
                            .unwrap();
                    PathBuf::from(core.path)
                } else {
                    show_error(app, anyhow!("No core for game").as_dyn_error(), true);
                    return;
                };

                // Load the core
                match app
                    .platform_mut()
                    .core_manager_mut()
                    .load_program(&core_path)
                {
                    Ok(mut core) => {
                        if let Err(e) = core.load_file(&file_path, None) {
                            show_error(
                                app,
                                anyhow!("Failed to load file: {}", e).as_dyn_error(),
                                true,
                            );
                        } else {
                            // Record in the Database the time we launched this game, ignoring
                            // errors.
                            let _ = game.play(&mut app.database().lock().unwrap());

                            run_core_loop(app, core, false);
                        }
                    }
                    Err(e) => {
                        show_error(
                            app,
                            anyhow!("Failed to load core: {}", e).as_dyn_error(),
                            true,
                        );
                        return;
                    }
                }
            }
            MenuAction::ShowDetails(i) => {
                let mut game = &mut all_games[i];
                if let Err(e) = details::games_details(app, game) {
                    show_error(app, e.as_dyn_error(), true);
                }
            }
            MenuAction::ChangeSort => sort_order = sort_order.next(),
        }
    }
}
