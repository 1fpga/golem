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
use retronomicon_dto::cores::CoreListItem;
use tracing::{debug, info};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum MenuAction {
    #[default]
    Back,

    ToggleCore(usize, bool),
    DoInstall,
}

impl MenuReturn for MenuAction {
    fn back() -> Self {
        MenuAction::Back
    }
}

impl SelectValue for MenuAction {
    fn next(&self) -> Self {
        match self {
            Self::ToggleCore(a, b) => Self::ToggleCore(*a, !*b),
            _ => Self::Back,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::ToggleCore(_, true) => "[X]",
            Self::ToggleCore(_, false) => "[ ]",
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
    let mut db = db_connection.write().unwrap();

    if mister_db::models::Core::has(&mut db, &core.slug, &core.latest_release.version)? {
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
        let _ = mister_db::models::Core::create(&mut db, &core, &core.latest_release, &core_path)?;
    }

    Ok(())
}

fn install_cores(app: &mut impl Application<Color = BinaryColor>, cores: Vec<&CoreListItem>) {
    for core in cores {
        if let Err(e) = install_single_core(app, &core) {
            show_error(app, &e.to_string());
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
    let mut should_install = cores
        .iter()
        .map(|core| {
            let path = paths::core_root(core, &core.latest_release);
            path.exists()
        })
        .collect::<Vec<_>>();

    let mut core_list = cores
        .iter()
        .enumerate()
        .map(|(i, core)| {
            let core_name = format!("{} ({})", core.name, &core.latest_release.version);
            Select::new(core_name, MenuAction::ToggleCore(i, should_install[i]))
                .with_value_converter(std::convert::identity)
        })
        .collect::<Vec<_>>();

    let mut menu = Menu::with_style(" Download Cores", style::menu_style())
        .add_items(&mut core_list)
        .add_item(NavigationItem::new("Install", MenuAction::DoInstall).with_marker(">"))
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
                        should_install[slug] = i;
                        info!("Installing core {} version {}", slug, i);
                        // info!("Installing core {} version {}", slug, release_version);
                    }
                    MenuAction::DoInstall => {
                        install_cores(
                            app,
                            cores
                                .iter()
                                .enumerate()
                                .filter(|(i, _)| should_install[*i])
                                .map(|(_, core)| core)
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
