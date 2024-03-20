use cfg_if::cfg_if;
use std::env;
use std::path::PathBuf;

pub fn config_root_path() -> PathBuf {
    // On DE10-Nano, the configurations are in /media/fat/golem
    cfg_if! {
        if #[cfg(feature = "platform_de10")] {
            let p = PathBuf::from("/media/fat/golem");
        } else {
            let p = dirs::config_dir().unwrap().join("golem");
        }
    }

    if !p.exists() {
        std::fs::create_dir_all(&p).unwrap();
    }
    p
}

pub fn screenshots_root() -> PathBuf {
    let p = config_root_path().join("screenshots");
    if !p.exists() {
        std::fs::create_dir_all(&p).unwrap();
    }
    p
}

pub fn core_root_path() -> PathBuf {
    let p = config_root_path().join("cores");
    if !p.exists() {
        std::fs::create_dir_all(&p).unwrap();
    }
    p
}

pub fn savestates_root_path() -> PathBuf {
    let p = config_root_path().join("savestates");
    if !p.exists() {
        std::fs::create_dir_all(&p).unwrap();
    }
    p
}

pub fn sav_root_path() -> PathBuf {
    let p = config_root_path().join("saves");
    if !p.exists() {
        std::fs::create_dir_all(&p).unwrap();
    }
    p
}

pub fn savestates_path(core_name: &str) -> PathBuf {
    savestates_root_path().join(core_name)
}

pub fn sav_path(core_name: &str) -> PathBuf {
    sav_root_path().join(core_name)
}

pub fn core_root(core: &retronomicon_dto::cores::CoreListItem) -> PathBuf {
    core_root_path().join(&core.slug)
}

pub fn core_release_root(
    core: &retronomicon_dto::cores::CoreListItem,
    release: &retronomicon_dto::cores::releases::CoreReleaseRef,
) -> PathBuf {
    core_root(core).join(&release.version)
}

pub fn settings_path() -> PathBuf {
    config_root_path().join("settings.json5")
}

pub fn all_settings_paths() -> Vec<PathBuf> {
    let mut paths = vec![config_root_path()];

    if let Some(mut home) = dirs::home_dir() {
        home.push(".mister");
        paths.push(home);
    }
    if let Ok(pwd_settings) = env::current_dir() {
        paths.push(pwd_settings);
    }
    if let Some(exec_settings) = dirs::executable_dir() {
        paths.push(exec_settings);
    }

    paths
        .into_iter()
        .flat_map(|p| {
            vec![
                p.join("MiSTer.json"),
                p.join("mister.json"),
                p.join("MiSTer.json5"),
                p.join("mister.json5"),
                p.join("settings.json"),
                p.join("settings.json5"),
            ]
        })
        .filter(|p| p.exists())
        .collect()
}
