use crate::application::menu::style;
use crate::application::TopLevelViewType;
use crate::macguiver::application::Application;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use embedded_menu::items::NavigationItem;
use embedded_menu::Menu;
use tracing::{error, info};

#[derive(Debug, Clone, Copy)]
pub enum MenuAction {
    Back,
    Refresh,
    ShowCoreInfo(i32),
}

fn build_cores_items_(database: &mut mister_db::Connection) -> Vec<(String, i32)> {
    use mister_db::diesel::prelude::*;
    use mister_db::models::*;
    use mister_db::schema::cores::dsl::*;

    let all_cores = cores.select(Core::as_select()).load(database);
    let all_cores = match all_cores {
        Ok(all_cores) => all_cores,
        Err(e) => {
            error!("Database error: {e}");
            return Vec::new();
        }
    };

    all_cores
        .iter()
        .map(|core| (core.name.clone(), core.id))
        .collect()
}

pub fn cores_menu_panel(app: &mut impl Application<Color = BinaryColor>) -> TopLevelViewType {
    let all_items = build_cores_items_(&mut app.database().write().unwrap());
    let mut items: Vec<NavigationItem<_, _, _, _>> = all_items
        .into_iter()
        .map(|(name, id)| NavigationItem::new(name, MenuAction::ShowCoreInfo(id)).with_marker(">"))
        .collect();

    let mut menu = Menu::with_style("_.Cores", style::menu_style())
        .add_item(NavigationItem::new("Refresh Cores...", MenuAction::Refresh).with_marker(">"))
        .add_items(&mut items)
        .add_item(NavigationItem::new("Back", MenuAction::Back).with_marker(">"))
        .build();

    app.event_loop(|app, state| {
        let buffer = app.main_buffer();
        buffer.clear(BinaryColor::Off).unwrap();
        menu.update(buffer);
        menu.draw(buffer).unwrap();

        for ev in state.events() {
            match menu.interact(ev) {
                None => {}
                Some(MenuAction::Refresh) => {
                    info!("Refreshing cores...");
                }
                Some(MenuAction::Back) => {
                    return Some(TopLevelViewType::MainMenu);
                }
                Some(MenuAction::ShowCoreInfo(id)) => {
                    info!("Showing core info for core {}", id);
                }
            }
        }

        None
    })
}

// impl Panel for CoreMenu {
//     fn new(_settings: &Settings, database: Arc<RwLock<mister_db::Connection>>) -> Self {
//         // Build the items
//         let all_items = build_cores_items_(&mut database.write().unwrap());
//         let mut items: Vec<NavigationItem<_>> = all_items
//             .into_iter()
//             .map(|(name, id)| {
//                 NavigationItem::new(name, MenuAction::ShowCoreInfo(id)).with_marker(">")
//             })
//             .collect();
//
//         let menu = Menu::with_style("Cores", style::menu_style())
//             .add_item(Chain::new(OwnedMenuItems::new(items)))
//             .add_item(NavigationItem::new("Refresh Cores...", MenuAction::Refresh).with_marker(">"))
//             .add_item(NavigationItem::new("Back", MenuAction::Back).with_marker(">"))
//             .build();
//
//         let menu = Rc::new(RefCell::new(menu));
//         let (update, draw) = {
//             let menu_update = menu.clone();
//             let menu_draw = menu.clone();
//
//             let update = move |state: &PlatformState| {
//                 let mut menu = menu_update.borrow_mut();
//                 menu.update(state);
//
//                 for ev in state.events() {
//                     if let Some(panel) = menu.interact(ev) {
//                         return Some(panel);
//                     }
//                 }
//
//                 return Ok(None);
//             };
//
//             let draw = move |target: &mut DrawBuffer<BinaryColor>| {
//                 let menu = menu_draw.borrow();
//                 menu.draw(target).unwrap();
//             };
//
//             (update, draw)
//         };
//
//         Self {
//             items: items,
//             update: Box::new(update),
//             draw: Box::new(draw),
//         }
//     }
//
//     fn update(&mut self, state: &PlatformState) -> Result<Option<TopLevelViewType>, String> {
//         let action = (self.update)(state)?;
//         if let Some(action) = action {
//             match action {
//                 MenuAction::Back => Ok(Some(TopLevelViewType::MainMenu)),
//                 MenuAction::Refresh => Ok(Some(TopLevelViewType::KeyboardTester)),
//                 MenuAction::ShowCoreInfo(_i) => Ok(Some(TopLevelViewType::KeyboardTester)),
//             }
//         } else {
//             Ok(None)
//         }
//     }
//
//     fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
//         (self.draw)(target)
//     }
// }
