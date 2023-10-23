use crate::application::menu::cores::download::cores_download_panel;
use crate::application::menu::filesystem::{select_file_path_menu, FilesystemMenuOptions};
use crate::application::menu::style::MenuReturn;
use crate::application::menu::text_menu;
use crate::application::panels::alert::show_error;
use crate::application::panels::core_loop::run_core_loop;
use crate::macguiver::application::Application;
use crate::platform::{CoreManager, MiSTerPlatform};
use embedded_graphics::pixelcolor::BinaryColor;
use tracing::{error, info};

#[derive(Default, Debug, Clone, Copy)]
pub enum MenuAction {
    #[default]
    Back,
    Manage,
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

pub fn cores_menu_panel(app: &mut impl Application<Color = BinaryColor>) {
    let mut state = None;

    loop {
        let all_cores = build_cores_items_(&mut app.database().write().unwrap());

        let mut menu_items = all_cores
            .iter()
            .enumerate()
            .map(|(i, core)| (core.name.as_str(), "", MenuAction::ShowCoreInfo(i)))
            .collect::<Vec<_>>();

        menu_items.push(("Load Core Manually", "", MenuAction::ManualLoad));
        menu_items.push(("Manage Cores", "", MenuAction::Manage));
        menu_items.push(("Back", "", MenuAction::Back));

        let (result, new_state) = text_menu(app, "Cores", &menu_items, state);
        state = Some(new_state);
        match result {
            MenuAction::Back => break,
            MenuAction::Manage => {
                cores_download_panel(app);
            }
            MenuAction::ShowCoreInfo(i) => {
                let core = &all_cores[i];
                let path = &core.path;
                info!("Loading core from path {:?}", path);

                app.hide_toolbar();
                let manager = app.platform_mut().core_manager_mut();

                let core = match manager.load_program(std::path::Path::new(&path)) {
                    Ok(core) => core,
                    Err(e) => {
                        show_error(app, format!("Failed to load core: {}", e));
                        return;
                    }
                };

                run_core_loop(app, core);
            }
            MenuAction::ManualLoad => {
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

                    // TODO: reload the Menu core here.
                } else {
                    info!("No core selected.");
                }
            }
        }
    }
}
