use crate::config;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

fn strip_version(name: &str) -> &str {
    name.rsplit_once('_').map(|(name, _)| name).unwrap_or(name)
}

fn core_name_of_(path: &Path) -> Option<&str> {
    Some(strip_version(exact_core_name_of_(path)?))
}

fn exact_core_name_of_(path: &Path) -> Option<&str> {
    if path.extension().and_then(OsStr::to_str) != Some("rbf") {
        return None;
    }

    path.file_stem()?.to_str()
}

fn find_core_(path: &Path, name: &str) -> Result<Option<PathBuf>, std::io::Error> {
    let stripped_name = strip_version(name);

    for entry in path.read_dir()? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let file_name = path.file_name().and_then(OsStr::to_str);
            if file_name.is_none() || !file_name.unwrap().starts_with('_') {
                continue;
            }
            if let Some(core) = find_core_(&path, name)? {
                return Ok(Some(core));
            }
        } else {
            let current_path = path.file_stem().and_then(OsStr::to_str);
            if let Some(current_path) = current_path {
                if strip_version(current_path) == stripped_name {
                    return Ok(Some(entry.path()));
                }
            }
        }
    }
    Ok(None)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreInfo {
    name: String,
    path: PathBuf,
}

impl CoreInfo {
    pub fn new(name: impl ToString, path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.to_string(),
            path: path.into(),
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        let path = path.as_ref();
        let name = core_name_of_(path)?;
        Some(Self::new(name, path))
    }

    pub fn from_name(
        name: impl AsRef<str>,
        core_root_dir: impl AsRef<Path>,
    ) -> Result<Option<Self>, std::io::Error> {
        let name = name.as_ref();
        let path = find_core_(core_root_dir.as_ref(), name)?;

        if let Some(path) = path {
            // Override name with core_name_of, to make sure we remove the version number if any.
            let name = core_name_of_(&path);
            Ok(name.map(|name| (Self::new(name, &path))))
        } else {
            Ok(None)
        }
    }

    pub fn from_exact_name(
        name: impl AsRef<str>,
        core_root_dir: impl AsRef<Path>,
    ) -> Result<Option<Self>, std::io::Error> {
        let name = name.as_ref();
        let path = find_core_(core_root_dir.as_ref(), name)?;

        if let Some(path) = path {
            // Check that the exact name is the same.
            if exact_core_name_of_(&path) != Some(name) {
                return Ok(None);
            }

            // Override name with core_name_of, to make sure we remove the version number if any.
            let name = core_name_of_(&path);
            Ok(name.map(|name| (Self::new(name, &path))))
        } else {
            Ok(None)
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn exact_name(&self) -> &str {
        exact_core_name_of_(&self.path).unwrap()
    }
}

impl From<config::BootCoreConfig> for Option<CoreInfo> {
    fn from(value: config::BootCoreConfig) -> Self {
        match value {
            config::BootCoreConfig::None => None,
            config::BootCoreConfig::LastCore => {
                let last_core_name = config::Config::last_core_data()?;
                Some(CoreInfo::from_name(last_core_name, config::Config::cores_root()).ok()??)
            }
            config::BootCoreConfig::ExactLastCore => {
                let last_core_name = config::Config::last_core_data()?;
                Some(
                    CoreInfo::from_exact_name(last_core_name, config::Config::cores_root())
                        .ok()??,
                )
            }
            config::BootCoreConfig::CoreName(name) => {
                Some(CoreInfo::from_name(name, config::Config::cores_root()).ok()??)
            }
        }
    }
}

#[test]
fn from_path_works() {
    let core = CoreInfo::from_path("config/cores/Somecore_12345678.rbf").unwrap();
    assert_eq!(core.name, "Somecore");
    assert_eq!(core.path, Path::new("config/cores/Somecore_12345678.rbf"));
}

#[test]
fn from_path_works_none() {
    let core = CoreInfo::from_path("config/cores/Somecore_12345678");
    assert_eq!(core, None);
}

#[test]
fn from_path_works_exact() {
    let core = CoreInfo::from_path("config/cores/Somecore.rbf").unwrap();
    assert_eq!(core.name, "Somecore");
    assert_eq!(core.path, Path::new("config/cores/Somecore.rbf"));
}

#[test]
fn from_name_works() {
    let root_dir = tempdir::TempDir::new("mister").unwrap();
    let root = root_dir.path();

    // Create a structure like this:
    //   mister/
    //   └── config/
    //       └── Core_12345678.rbf
    //       └── Wrong_12345678.rbf
    //   └── _Cores/
    //       └── Core_12345678.rbf
    //       └── hello.rbf
    //   └── _Other/
    //       └── Other_12345678.rbf
    //       └── _Again/
    //           └── Bar_12345678.rbf

    std::fs::create_dir_all(root.join("config")).unwrap();
    std::fs::create_dir_all(root.join("_Cores")).unwrap();
    std::fs::create_dir_all(root.join("_Other")).unwrap();
    std::fs::create_dir_all(root.join("_Other/_Again")).unwrap();
    std::fs::write(root.join("config/Core_12345678.rbf"), "").unwrap();
    std::fs::write(root.join("_Cores/Core_12345678.rbf"), "").unwrap();
    std::fs::write(root.join("_Cores/hello.rbf"), "").unwrap();
    std::fs::write(root.join("_Other/Other_12345678.rbf"), "").unwrap();
    std::fs::write(root.join("_Other/_Again/Bar_12345678.rbf"), "").unwrap();

    // Testing base case.
    // Core_12345678 should be in $root/_Cores/ (not $root/config/).
    let core = CoreInfo::from_name("Core", root).unwrap().unwrap();
    assert_eq!(core.name, "Core");
    assert_eq!(core.path, root.join("_Cores/Core_12345678.rbf"));

    // Testing no version number.
    // hello should be in $root/_Cores/ (not $root/config/).
    let core = CoreInfo::from_name("hello", root).unwrap().unwrap();
    assert_eq!(core.name, "hello");
    assert_eq!(core.path, root.join("_Cores/hello.rbf"));

    // Testing iterative over directories.
    // Other_12345678 should be in $root/_Other/ (not $root/config/).
    let core = CoreInfo::from_name("Other", root).unwrap().unwrap();
    assert_eq!(core.name, "Other");
    assert_eq!(core.path, root.join("_Other/Other_12345678.rbf"));

    // Testing recursive over directories.
    // Other_12345678 should be in $root/_Other/ (not $root/config/).
    let core = CoreInfo::from_name("Bar", root).unwrap().unwrap();
    assert_eq!(core.name, "Bar");
    assert_eq!(core.path, root.join("_Other/_Again/Bar_12345678.rbf"));

    // Testing skipping directories not starting with `_`.
    let core = CoreInfo::from_name("Wrong", root).unwrap();
    assert_eq!(core, None);
}

#[test]
fn from_bootcore_config() {
    let root_dir = tempdir::TempDir::new("mister").unwrap();
    let root = root_dir.path();
    config::testing::set_config_root(root);

    // Create a structure like this:
    //   mister/
    //   └── config/
    //       └── lastcore.dat (contains "Core")
    //       └── Core_12345678.rbf
    //       └── Wrong_12345678.rbf
    //   └── _Cores/
    //       └── Core_12345678.rbf
    //       └── hello.rbf
    //   └── _Other/
    //       └── Other_12345678.rbf
    //       └── _Again/
    //           └── Bar_12345678.rbf

    std::fs::create_dir_all(root.join("config")).unwrap();
    std::fs::create_dir_all(root.join("_Cores")).unwrap();
    std::fs::create_dir_all(root.join("_Other")).unwrap();
    std::fs::create_dir_all(root.join("_Other/_Again")).unwrap();
    std::fs::write(root.join("config/Core_12345678.rbf"), "").unwrap();
    std::fs::write(root.join("config/lastcore.dat"), "Core").unwrap();
    std::fs::write(root.join("_Cores/Core_12345678.rbf"), "").unwrap();
    std::fs::write(root.join("_Cores/hello.rbf"), "").unwrap();
    std::fs::write(root.join("_Other/Other_12345678.rbf"), "").unwrap();
    std::fs::write(root.join("_Other/_Again/Bar_12345678.rbf"), "").unwrap();

    let x: Option<CoreInfo> = Option::<CoreInfo>::from(config::BootCoreConfig::LastCore);
    let core = x.unwrap();
    assert_eq!(core.name, "Core");
    assert_eq!(core.path, root.join("_Cores/Core_12345678.rbf"));
}
