use crate::application::menu::style;
use crate::application::{Panel, TopLevelViewType};
use crate::data::settings::Settings;
use crate::macguiver::buffer::DrawBuffer;
use crate::platform::PlatformState;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use embedded_layout::object_chain::Chain;
use embedded_menu::interaction::InteractionType;
use embedded_menu::items::NavigationItem;
use embedded_menu::Menu;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use tracing::error;

#[derive(Debug, Clone, Copy)]
pub enum MenuAction {
    Back,
    Refresh,
    ShowCoreInfo(i32),
}

type BoxedUpdateFn = Box<dyn FnMut(&PlatformState) -> Result<Option<MenuAction>, String>>;
type BoxedDrawFn = Box<dyn Fn(&mut DrawBuffer<BinaryColor>)>;

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

pub struct CoreMenu {
    // Function to update the menu.
    update: BoxedUpdateFn,

    // Function to draw the menu.
    draw: BoxedDrawFn,
}

impl Panel for CoreMenu {
    fn new(_settings: &Settings, database: Arc<RwLock<mister_db::Connection>>) -> Self {
        // Build the items
        let all_items = build_cores_items_(&mut database.write().unwrap());
        let mut items: Vec<NavigationItem<_>> = all_items
            .into_iter()
            .map(|(name, id)| {
                NavigationItem::new(name, MenuAction::ShowCoreInfo(id)).with_marker(">")
            })
            .collect();

        let menu = Menu::with_style("Cores", style::menu_style())
            .add_item(Chain::new(OwnedMenuItems::new(items)))
            .add_item(NavigationItem::new("Refresh Cores...", MenuAction::Refresh).with_marker(">"))
            .add_item(NavigationItem::new("Back", MenuAction::Back).with_marker(">"))
            .build();

        let menu = Rc::new(RefCell::new(menu));
        let (update, draw) = {
            let menu_update = menu.clone();
            let menu_draw = menu.clone();

            let update = move |state: &PlatformState| {
                let mut menu = menu_update.borrow_mut();
                menu.update(state);

                for ev in state.events() {
                    if let Some(panel) = menu.interact(ev) {
                        return Some(panel);
                    }
                }

                return Ok(None);
            };

            let draw = move |target: &mut DrawBuffer<BinaryColor>| {
                let menu = menu_draw.borrow();
                menu.draw(target).unwrap();
            };

            (update, draw)
        };

        Self {
            items: items,
            update: Box::new(update),
            draw: Box::new(draw),
        }
    }

    fn update(&mut self, state: &PlatformState) -> Result<Option<TopLevelViewType>, String> {
        let action = (self.update)(state)?;
        if let Some(action) = action {
            match action {
                MenuAction::Back => Ok(Some(TopLevelViewType::MainMenu)),
                MenuAction::Refresh => Ok(Some(TopLevelViewType::KeyboardTester)),
                MenuAction::ShowCoreInfo(_i) => Ok(Some(TopLevelViewType::KeyboardTester)),
            }
        } else {
            Ok(None)
        }
    }

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
        (self.draw)(target)
    }
}
