use crate::application::menu::games::manage::manage_games;
use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::application::panels::alert::{alert, show_error};
use crate::application::panels::core_loop::run_core_loop;
use crate::macguiver::application::Application;
use crate::platform::{Core, CoreManager, GoLEmPlatform};
use embedded_graphics::pixelcolor::BinaryColor;
use golem_db::models::GameOrder;
use std::path::PathBuf;

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
                .with_suffix(&[("Manage Games", "", MenuAction::Manage).into()])
                .with_details("Details"),
        );
        state = Some(new_state);

        match result {
            MenuAction::Back => break,
            MenuAction::Manage => manage_games(app),
            MenuAction::LoadGame(i) => {
                let game = &all_games[i];
                let file_path = match game.path.as_ref() {
                    Some(path) => path,
                    None => {
                        show_error(app, "No file path for game");
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
                    show_error(app, "No core for game");
                    return;
                };

                // Load the core
                match app
                    .platform_mut()
                    .core_manager_mut()
                    .load_program(&core_path)
                {
                    Ok(core) => {
                        if let Err(e) = core.load_file(&file_path) {
                            show_error(app, format!("Failed to load file: {}", e));
                        } else {
                            run_core_loop(app, core);
                        }
                    }
                    Err(e) => {
                        show_error(app, format!("Failed to load core: {}", e));
                        return;
                    }
                }

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
