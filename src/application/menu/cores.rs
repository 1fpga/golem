use crate::application::menu::filesystem::{select_file_path_menu, FilesystemMenuOptions};
use crate::application::menu::style;
use crate::application::menu::style::MenuReturn;
use crate::application::TopLevelViewType;
use crate::macguiver::application::Application;
use crate::platform::{CoreManager, MiSTerPlatform};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use embedded_menu::items::NavigationItem;
use embedded_menu::Menu;
use tracing::{error, info};

#[derive(Default, Debug, Clone, Copy)]
pub enum MenuAction {
    #[default]
    Back,
    Refresh,
    ManualLoad,
    ShowCoreInfo(i32),
}

impl MenuReturn for MenuAction {
    fn back() -> Self {
        MenuAction::Back
    }
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
        .add_item(
            NavigationItem::new("Load Core Manually", MenuAction::ManualLoad).with_marker(">"),
        )
        .add_item(NavigationItem::new("Refresh Cores", MenuAction::Refresh))
        .add_items(&mut items)
        .add_item(NavigationItem::new("Back", MenuAction::Back).with_marker(">"))
        .build();

    app.event_loop(|app, state| {
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
                Some(MenuAction::ManualLoad) => {
                    let path = select_file_path_menu(
                        app,
                        "Select Core Manually",
                        std::env::current_exe()
                            .unwrap()
                            .parent()
                            .unwrap()
                            .to_path_buf(),
                        FilesystemMenuOptions::default().with_allow_back(true),
                    );
                    info!("Loading core from path {:?}", path);

                    if let Ok(Some(path)) = path {
                        let core = app
                            .platform_mut()
                            .core_manager_mut()
                            .load_program(path)
                            .expect("Failed to load core");
                    } else {
                        info!("No core selected.");
                    }
                }
            }
        }

        let buffer = app.main_buffer();
        buffer.clear(BinaryColor::Off).unwrap();
        menu.update(buffer);
        menu.draw(buffer).unwrap();

        None
    })
}
