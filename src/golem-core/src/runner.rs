use crate::{CoreManager, GolemCore};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
enum CoreLoader {
    RbfFile(PathBuf),
    Menu,
}

impl CoreLoader {
    pub(crate) fn load(&self, manager: &mut CoreManager) -> Result<GolemCore, String> {
        match self {
            CoreLoader::RbfFile(rbf_path) => manager.load_core(rbf_path),
            CoreLoader::Menu => manager.load_menu(),
        }
    }
}

#[derive(Debug, Default, Clone)]
enum Slot {
    #[default]
    Empty,
    File(PathBuf),
}

#[derive(Debug, Clone)]
pub struct CoreLauncher {
    core_loader: CoreLoader,
    files: Vec<PathBuf>,
    sav: Slot,
    savestate: Slot,
    slotted_savestate: HashMap<usize, Slot>,
}

impl CoreLauncher {
    fn new(core_loader: CoreLoader) -> Self {
        Self {
            core_loader,
            files: Default::default(),
            sav: Default::default(),
            savestate: Default::default(),
            slotted_savestate: Default::default(),
        }
    }

    pub fn rbf(rbf_path: PathBuf) -> Self {
        Self::new(CoreLoader::RbfFile(rbf_path))
    }

    pub fn menu() -> Self {
        Self::new(CoreLoader::Menu)
    }

    pub fn with_sav(mut self, file: PathBuf) -> Self {
        self.sav = Slot::File(file);
        self
    }

    pub fn with_savestate_slot(mut self, slot: usize, file: PathBuf) -> Self {
        self.slotted_savestate.insert(slot, Slot::File(file));
        self
    }

    pub fn with_savestate(mut self, file: PathBuf) -> Self {
        self.savestate = Slot::File(file);
        self
    }

    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.files.push(file);
        self
    }

    pub fn launch(self, manager: &mut CoreManager) -> Result<GolemCore, String> {
        let mut core = self.core_loader.load(manager)?;

        if !self.files.is_empty() {
            let should_sav = core
                .menu_options()
                .iter()
                .filter_map(|x| x.as_load_file_info())
                .any(|i| i.save_support);

            for file in self.files {
                core.load_file(&file, None)?;
            }

            if should_sav {
                if let Slot::File(ref file) = self.sav {
                    core.mount_sav(file)?;
                }
            }
            core.end_send_file()?;
            core.check_sav()?;
        }

        // Load all savestates.
        if let Some(savestate_manager) = core.save_states() {
            for (slot, state) in savestate_manager.iter_mut().enumerate() {
                let path = match self.slotted_savestate.get(&slot) {
                    Some(Slot::File(ref path)) => path,
                    _ => continue,
                };
                let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
                state.read_from(f)?;
            }
        }

        Ok(core)
    }
}
