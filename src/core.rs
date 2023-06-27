use std::ffi::OsStr;
use std::path::{Path, PathBuf};

fn core_name_of_(path: &Path) -> Option<&str> {
    if path.extension().and_then(OsStr::to_str) != Some("rbf") {
        return None;
    }

    path.file_name()?
        .to_str()?
        .rsplit_once('_')
        .map(|(name, _)| name.into())
        .or_else(|| path.file_stem().and_then(OsStr::to_str))
}

fn find_core_(path: &Path, name: &str) -> Result<Option<PathBuf>, std::io::Error> {
    for entry in path.read_dir()? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let file_name = path.file_name().and_then(OsStr::to_str);
            if file_name == None || !file_name.unwrap().starts_with('_') {
                continue;
            }
            if let Some(core) = find_core_(&path, name)? {
                eprintln!("find_core_().1 == {:?}", core);
                return Ok(Some(core));
            }
        } else if path
            .file_stem()
            .and_then(OsStr::to_str)
            .map(|n| n.starts_with(name))
            .unwrap_or(false)
        {
            return Ok(Some(entry.path()));
        }
    }
    Ok(None)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Core {
    pub(super) name: String,
    pub(super) path: PathBuf,
}

impl Core {
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
        let path = find_core_(core_root_dir.as_ref(), name.as_ref())?;

        if let Some(path) = path {
            // Override name with core_name_of, to make sure we remove the version number if any.
            let name = core_name_of_(&path);
            Ok(name.map(|name| (Self::new(name, &path))))
        } else {
            Ok(None)
        }
    }
}

#[test]
fn from_path_works() {
    let core = Core::from_path("config/cores/Somecore_12345678.rbf").unwrap();
    assert_eq!(core.name, "Somecore");
    assert_eq!(core.path, Path::new("config/cores/Somecore_12345678.rbf"));
}

#[test]
fn from_path_works_none() {
    let core = Core::from_path("config/cores/Somecore_12345678");
    assert_eq!(core, None);
}

#[test]
fn from_path_works_exact() {
    let core = Core::from_path("config/cores/Somecore.rbf").unwrap();
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
    let core = Core::from_name("Core", root).unwrap().unwrap();
    assert_eq!(core.name, "Core");
    assert_eq!(core.path, root.join("_Cores/Core_12345678.rbf"));

    // Testing no version number.
    // hello should be in $root/_Cores/ (not $root/config/).
    let core = Core::from_name("hello", root).unwrap().unwrap();
    assert_eq!(core.name, "hello");
    assert_eq!(core.path, root.join("_Cores/hello.rbf"));

    // Testing iterative over directories.
    // Other_12345678 should be in $root/_Other/ (not $root/config/).
    let core = Core::from_name("Other", root).unwrap().unwrap();
    assert_eq!(core.name, "Other");
    assert_eq!(core.path, root.join("_Other/Other_12345678.rbf"));

    // Testing recursive over directories.
    // Other_12345678 should be in $root/_Other/ (not $root/config/).
    let core = Core::from_name("Bar", root).unwrap().unwrap();
    assert_eq!(core.name, "Bar");
    assert_eq!(core.path, root.join("_Other/_Again/Bar_12345678.rbf"));
}
