use crate::application::menu::style;
use crate::application::menu::style::MenuReturn;
use crate::application::panels::alert::show_error;
use crate::data::paths;
use crate::macguiver::application::Application;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use embedded_menu::items::select::SelectValue;
use embedded_menu::items::{NavigationItem, Select};
use embedded_menu::Menu;
use golem_db::models;
use retronomicon_dto::cores::CoreListItem;
use tracing::{debug, info};

/// The action to perform for a selected core.
#[derive(Debug, Clone, Copy, PartialEq)]
enum CoreAction {
    /// The core is not installed.
    IsNotInstalled,
    /// The core should be installed.
    Install,

    /// The core is already installed.
    IsInstalled,

    /// The core should be uninstalled.
    Uninstall,
}

impl CoreAction {
    pub fn toggle(&self) -> Self {
        match self {
            Self::IsNotInstalled => Self::Install,
            Self::Install => Self::IsNotInstalled,

            Self::IsInstalled => Self::Uninstall,
            Self::Uninstall => Self::IsInstalled,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CoreAction::IsNotInstalled => "",
            CoreAction::Install => "Install",
            CoreAction::IsInstalled => "Installed",
            CoreAction::Uninstall => "Remove",
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
enum MenuAction {
    #[default]
    Back,

    ToggleCore(usize, CoreAction),
    /// Execute the selected actions for all cores.
    Execute,
}

impl MenuReturn for MenuAction {
    fn back() -> Option<Self> {
        Some(MenuAction::Back)
    }
}

impl SelectValue for MenuAction {
    fn next(&self) -> Self {
        match self {
            Self::ToggleCore(a, b) => Self::ToggleCore(*a, b.toggle()),
            _ => Self::Back,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::ToggleCore(_, action) => action.as_str(),
            _ => "",
        }
    }
}

fn list_cores_from_retronomicon(
    app: &mut impl Application<Color = BinaryColor>,
) -> Result<Vec<retronomicon_dto::cores::CoreListItem>, anyhow::Error> {
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let backend = app.settings().retronomicon_backend();
    let url_base = backend
        .first()
        .ok_or(anyhow::Error::msg("No Retronomicon backend"))?;

    let url = url_base.join("cores?platform=mister-fpga")?;
    debug!(url = url.to_string(), "Download core list");
    let response = client.get(url).send()?;

    Ok(response.json()?)
}

fn install_single_core(
    app: &mut impl Application<Color = BinaryColor>,
    core: &CoreListItem,
) -> Result<(), anyhow::Error> {
    let client = reqwest::blocking::Client::builder()
        .tls_built_in_root_certs(true)
        .build()
        .unwrap();
    let db_connection = app.database();
    let mut db = db_connection.lock().unwrap();

    if golem_db::models::Core::has(&mut db, &core.slug, &core.latest_release.version)? {
        info!(?core, "Core already installed");
        return Ok(());
    }

    info!(?core, "Installing core");

    let artifacts: Vec<retronomicon_dto::artifact::CoreReleaseArtifactListItem> = client
        .get(format!(
            "https://alpha.retronomicon.land/api/v1/cores/{}/releases/{}/artifacts",
            core.slug, core.latest_release.id
        ))
        .send()?
        .json()?;

    for a in artifacts.iter() {
        let file = client
            .get(format!(
                "https://alpha.retronomicon.land/{}",
                a.download_url.as_ref().unwrap()
            ))
            .send()?
            .bytes()?;

        let core_root = paths::core_root(core, &core.latest_release);
        let core_path = core_root.join(&a.filename);
        std::fs::create_dir_all(&core_root)?;

        std::fs::write(&core_path, file)?;
        // Update the database.
        let _ = golem_db::models::Core::create(&mut db, &core, &core.latest_release, &core_path)?;
    }

    Ok(())
}

fn execute_core_actions(
    app: &mut impl Application<Color = BinaryColor>,
    cores: Vec<(&CoreListItem, CoreAction)>,
) {
    for (core, action) in cores {
        match action {
            CoreAction::Install => {
                if let Err(e) = install_single_core(app, &core) {
                    show_error(app, &e.to_string());
                }
            }
            CoreAction::Uninstall => {
                let core_root = paths::core_root(core, &core.latest_release);
                if let Err(e) = std::fs::remove_dir_all(&core_root) {
                    show_error(app, &e.to_string());
                } else {
                    let _ = models::Core::delete(&mut app.database().lock().unwrap(), core.id);
                }
            }
            _ => {}
        }
    }
}

pub fn cores_download_panel(app: &mut impl Application<Color = BinaryColor>) {
    let cores = match list_cores_from_retronomicon(app) {
        Ok(cores) => cores,
        Err(e) => {
            show_error(app, &e.to_string());
            return;
        }
    };
    let mut actions = cores
        .iter()
        .map(|core| {
            let path = paths::core_root(core, &core.latest_release);
            if path.exists() {
                CoreAction::IsInstalled
            } else {
                CoreAction::IsNotInstalled
            }
        })
        .collect::<Vec<_>>();

    let mut core_list = cores
        .iter()
        .enumerate()
        .map(|(i, core)| {
            let core_name = format!("{} ({})", core.name, &core.latest_release.version);
            Select::new(core_name, MenuAction::ToggleCore(i, actions[i]))
                .with_value_converter(std::convert::identity)
        })
        .collect::<Vec<_>>();

    let mut menu = Menu::with_style(" Download Cores", style::menu_style_simple())
        .add_items(&mut core_list)
        .add_item(
            NavigationItem::new("Install / Uninstall selected cores", MenuAction::Execute)
                .with_marker(">"),
        )
        .add_item(NavigationItem::new("Cancel", MenuAction::Back).with_marker(">"))
        .build();

    app.event_loop(move |app, state| {
        let buffer = app.main_buffer();
        buffer.clear(BinaryColor::Off).unwrap();
        menu.update(buffer);
        menu.draw(buffer).unwrap();

        for ev in state.events() {
            if let Some(action) = menu.interact(ev) {
                match action {
                    MenuAction::Back => return Some(()),
                    MenuAction::ToggleCore(slug, i) => {
                        actions[slug] = i;
                    }
                    MenuAction::Execute => {
                        execute_core_actions(
                            app,
                            cores
                                .iter()
                                .enumerate()
                                .filter_map(|(i, core)| match actions[i] {
                                    a @ CoreAction::Install | a @ CoreAction::Uninstall => {
                                        Some((core, a))
                                    }
                                    _ => None,
                                })
                                .collect(),
                        );
                        // TODO: uninstall cores.
                        return Some(());
                    }
                }
            }
        }

        None
    })
}
