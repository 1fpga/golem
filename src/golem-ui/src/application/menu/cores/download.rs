use crate::application::menu::style;
use crate::application::menu::style::MenuReturn;
use crate::application::panels::alert::show_error;
use crate::application::GoLEmApp;
use crate::data::paths;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use embedded_menu::items::select::SelectValue;
use embedded_menu::items::{NavigationItem, Select};
use embedded_menu::Menu;
use golem_db::models;
use retronomicon_dto::cores::CoreListItem;
use retronomicon_dto::routes;
use thiserror::__private::AsDynError;
use tracing::{debug, info};

/// For now, we don't allow cores that are known to not work at all to be
/// installed.
const SAFE_LIST: &'static [&'static str] = &[
    "mister-input-test",
    "mister-memtest",
    "mister-nes",
    "mister-chess",
];

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

fn list_cores_from_retronomicon(app: &mut GoLEmApp) -> Result<Vec<CoreListItem>, anyhow::Error> {
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let backend = app.settings().retronomicon_backend();
    let url_base = backend
        .first()
        .ok_or(anyhow::Error::msg("No Retronomicon backend"))?;

    let mut url = routes::v1::cores(url_base);
    url.query_pairs_mut().append_pair("platform", "mister-fpga");
    debug!(url = url.to_string(), "Download core list");
    let response = client.get(url).send()?;
    let list: Vec<CoreListItem> = response.json()?;
    Ok(list
        .into_iter()
        .filter(|c| SAFE_LIST.contains(&c.slug.as_str()))
        .collect())
}

fn install_single_core(app: &mut GoLEmApp, core: &CoreListItem) -> Result<(), anyhow::Error> {
    let client = reqwest::blocking::Client::builder()
        .tls_built_in_root_certs(true)
        .build()
        .unwrap();

    let backend = if let Some(backend) = app.settings().retronomicon_backend().first() {
        backend.clone()
    } else {
        return Err(anyhow::Error::msg("No Retronomicon backend"));
    };
    let db_connection = app.database();
    let mut db = db_connection.lock().unwrap();

    let latest_release = if let Some(latest_release) = &core.latest_release {
        if models::Core::has(&mut db, &core.slug, &latest_release.version)? {
            info!(?core, "Core already installed");
            return Ok(());
        }
        latest_release
    } else {
        info!(?core, "Core has no releases");
        return Ok(());
    };

    info!(?core, "Installing core");
    let artifacts: Vec<retronomicon_dto::artifact::CoreReleaseArtifactListItem> = client
        .get(routes::v1::cores_releases_artifacts(
            &backend,
            &core.slug.as_str().into(),
            latest_release.id,
        ))
        .send()?
        .json()?;

    let rclient = reqwest::blocking::Client::builder().build()?;
    for a in artifacts.iter() {
        let file = rclient.get(&a.download_url).send()?.bytes()?;

        let core_root = paths::core_release_root(core, &latest_release);
        let core_path = core_root.join(&a.filename);
        std::fs::create_dir_all(&core_root)?;

        std::fs::write(&core_path, file)?;
        // Update the database.
        let _ = models::Core::create(&mut db, core, &core.system, latest_release, &core_path)?;
    }

    Ok(())
}

fn execute_core_actions(app: &mut GoLEmApp, cores: Vec<(&CoreListItem, CoreAction)>) {
    for (core, action) in cores {
        match action {
            CoreAction::Install => {
                if let Err(e) = install_single_core(app, &core) {
                    show_error(app, e.as_dyn_error(), true);
                }
            }
            CoreAction::Uninstall => {
                let core_root = paths::core_root(core);
                if let Err(e) = std::fs::remove_dir_all(&core_root) {
                    show_error(app, e.as_dyn_error(), true);
                } else {
                    let _ = models::Core::delete(&mut app.database().lock().unwrap(), core.id);
                }
            }
            _ => {}
        }
    }
}

pub fn cores_download_panel(app: &mut GoLEmApp) {
    let cores = match list_cores_from_retronomicon(app) {
        Ok(cores) => cores,
        Err(e) => {
            show_error(app, e.as_dyn_error(), true);
            return;
        }
    };
    let mut actions = cores
        .iter()
        .filter_map(|core| {
            let latest_release = if let Some(latest_release) = &core.latest_release {
                latest_release
            } else {
                return None;
            };
            let path = paths::core_release_root(core, &latest_release);
            if path.exists() {
                Some(CoreAction::IsInstalled)
            } else {
                Some(CoreAction::IsNotInstalled)
            }
        })
        .collect::<Vec<_>>();

    let mut core_list = cores
        .iter()
        .enumerate()
        .filter_map(|(i, core)| {
            let latest_release = if let Some(latest_release) = &core.latest_release {
                latest_release
            } else {
                return None;
            };

            let core_name = format!("{} ({})", core.name, &latest_release.version);
            Some(
                Select::new(core_name, MenuAction::ToggleCore(i, actions[i]))
                    .with_value_converter(std::convert::identity),
            )
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
