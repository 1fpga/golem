// use anyhow::anyhow;
// use thiserror::__private::AsDynError;
//
// use golem_db::models::GameOrder;
//
// use crate::application::coordinator::GameStartInfo;
// use crate::application::menu::games::manage::manage_games;
// use crate::application::menu::style::MenuReturn;
// use crate::application::menu::{text_menu, TextMenuOptions};
// use crate::application::panels::alert::show_error;
// use crate::application::panels::core_loop::run_core_loop;
// use crate::application::GoLEmApp;
//
// mod details;
// mod manage;
//
// #[derive(Default, Debug, Clone, Copy)]
// enum MenuAction {
//     #[default]
//     Back,
//     ChangeSort,
//     Manage,
//     LoadGame(usize),
//     ShowDetails(usize),
// }
//
// impl MenuReturn for MenuAction {
//     fn back() -> Option<Self> {
//         Some(MenuAction::Back)
//     }
//     fn into_details(self) -> Option<Self> {
//         match self {
//             MenuAction::LoadGame(i) => Some(MenuAction::ShowDetails(i)),
//             _ => None,
//         }
//     }
//     fn sort() -> Option<Self> {
//         Some(Self::ChangeSort)
//     }
// }
//
// fn build_games_list_(
//     database: &mut golem_db::Connection,
//     sort: GameOrder,
// ) -> Vec<golem_db::models::Game> {
//     let all_games = golem_db::models::Game::list(database, 0, 1000, sort);
//     all_games.unwrap_or_else(|e| {
//         tracing::error!("Database error: {e}");
//         Vec::new()
//     })
// }
//
// pub fn games_list(app: &mut GoLEmApp) {
//     let mut state = None;
//     let mut sort_order = GameOrder::LastPlayed;
//
//     loop {
//         let mut all_games = build_games_list_(&mut app.database().lock().unwrap(), sort_order);
//
//         let menu_items = all_games
//             .iter()
//             .enumerate()
//             .map(|(i, game)| (game.name.as_str(), "", MenuAction::LoadGame(i)))
//             .collect::<Vec<_>>();
//
//         let (result, new_state) = text_menu(
//             app,
//             "Games",
//             &menu_items,
//             TextMenuOptions::default()
//                 .with_state(state)
//                 .with_sort(sort_order.as_str())
//                 .with_suffix(&[("Manage Games", "", MenuAction::Manage).into()])
//                 .with_details("Details"),
//         );
//         state = Some(new_state);
//
//         match result {
//             MenuAction::Back => break,
//             MenuAction::Manage => manage_games(app),
//             MenuAction::LoadGame(i) => {
//                 let game = &mut all_games[i];
//
//                 let (should_show_menu, mut core) = match app.coordinator_mut().launch_game(
//                     app,
//                     GameStartInfo::default()
//                         .with_maybe_core_id(game.core_id)
//                         .with_game_id(game.id),
//                 ) {
//                     Ok((should_show_menu, core)) => (should_show_menu, core),
//                     Err(e) => {
//                         show_error(
//                             app,
//                             anyhow!("Failed to start game: {}", e).as_dyn_error(),
//                             true,
//                         );
//                         return;
//                     }
//                 };
//
//                 // Run the core loop.
//                 run_core_loop(app, &mut core, should_show_menu);
//             }
//             MenuAction::ShowDetails(i) => {
//                 let game = &mut all_games[i];
//                 if let Err(e) = details::games_details(app, game) {
//                     show_error(app, e.as_dyn_error(), true);
//                 }
//             }
//             MenuAction::ChangeSort => sort_order = sort_order.next(),
//         }
//     }
// }
