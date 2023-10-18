use crate::application::menu::cores::download::cores_download_panel;
use crate::application::menu::filesystem::{select_file_path_menu, FilesystemMenuOptions};
use crate::application::menu::style;
use crate::application::menu::style::MenuReturn;
use crate::application::panels::alert::show_error;
use crate::application::panels::core_loop::run_core_loop;
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
    Download,
    ManualLoad,
    ShowCoreInfo(usize),
}

impl MenuReturn for MenuAction {
    fn back() -> Self {
        MenuAction::Back
    }
}

fn build_cores_items_(database: &mut mister_db::Connection) -> Vec<mister_db::models::Core> {
    let all_cores = mister_db::models::Core::list(database);
    match all_cores {
        Ok(all_cores) => all_cores,
        Err(e) => {
            error!("Database error: {e}");
            Vec::new()
        }
    }
}

pub fn cores_menu_panel(app: &mut impl Application<Color = BinaryColor>) -> TopLevelViewType {
    let all_items = build_cores_items_(&mut app.database().write().unwrap());
    let mut items: Vec<NavigationItem<_, _, _, _>> = all_items
        .iter()
        .enumerate()
        .map(|(i, core)| {
            NavigationItem::new(&core.name, MenuAction::ShowCoreInfo(i)).with_marker(">")
        })
        .collect();

    let mut menu = Menu::with_style(" Cores", style::menu_style())
        .add_items(&mut items)
        .add_item(
            NavigationItem::new("Load Core Manually", MenuAction::ManualLoad).with_marker(">"),
        )
        .add_item(NavigationItem::new(
            "Install/Uninstall Cores",
            MenuAction::Download,
        ))
        .add_item(NavigationItem::new("Back", MenuAction::Back).with_marker(">"))
        .build();

    app.event_loop(|app, state| {
        for ev in state.events() {
            match menu.interact(ev) {
                None => {}
                Some(MenuAction::Download) => {
                    cores_download_panel(app);
                    return Some(TopLevelViewType::Cores);
                }
                Some(MenuAction::Back) => {
                    return Some(TopLevelViewType::MainMenu);
                }
                Some(MenuAction::ShowCoreInfo(i)) => {
                    let core = &all_items[i];
                    let path = &core.path;
                    info!("Loading core from path {:?}", path);

                    app.hide_toolbar();
                    let manager = app.platform_mut().core_manager_mut();

                    let core = match manager.load_program(std::path::Path::new(&path)) {
                        Ok(core) => core,
                        Err(e) => {
                            show_error(app, format!("Failed to load core: {}", e));
                            return Some(TopLevelViewType::Cores);
                        }
                    };

                    run_core_loop(app, core);
                }
                Some(MenuAction::ManualLoad) => {
                    let path = select_file_path_menu(
                        app,
                        "Select Core Manually",
                        std::env::current_exe().unwrap().parent().unwrap(),
                        FilesystemMenuOptions::default().with_allow_back(true),
                    );
                    info!("Loading core from path {:?}", path);

                    if let Ok(Some(path)) = path {
                        app.hide_toolbar();
                        let core = app
                            .platform_mut()
                            .core_manager_mut()
                            .load_program(&path)
                            .expect("Failed to load core");
                        run_core_loop(app, core);
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
