use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::core::{Bios, Rom};

/// The type of core to launch.
#[derive(Debug, Clone)]
pub enum CoreType {
    /// Don't launch a new core, keep the current one running.
    Current,

    /// Launch a core from an RBF file.
    RbfFile(PathBuf),

    /// Launch the menu core.
    Menu,
}

#[derive(Debug, Clone)]
pub enum Slot {
    File(PathBuf),
    Memory(PathBuf, Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct CoreLaunchInfo<T> {
    pub core: CoreType,
    pub rom: Option<Rom>,
    pub bios: Vec<Bios>,
    pub files: BTreeMap<usize, Slot>,
    pub save_state: Vec<Slot>,

    pub data: T,
}

impl CoreLaunchInfo<()> {
    fn new(core_loader: CoreType) -> Self {
        Self {
            core: core_loader,
            rom: None,
            bios: Default::default(),
            files: Default::default(),
            save_state: Default::default(),
            data: (),
        }
    }

    pub fn rbf(rbf_path: PathBuf) -> Self {
        Self::new(CoreType::RbfFile(rbf_path))
    }

    pub fn menu() -> Self {
        Self::new(CoreType::Menu)
    }

    pub fn current() -> Self {
        Self::new(CoreType::Current)
    }
}

impl<T> CoreLaunchInfo<T> {
    pub fn with_rom(mut self, rom: Rom) -> Self {
        self.rom = Some(rom);
        self
    }

    pub fn with_file(mut self, slot: usize, content: Slot) -> Self {
        self.files.insert(slot, content);
        self
    }

    pub fn with_save_state(mut self, content: Slot) -> Self {
        self.save_state.push(content);
        self
    }

    pub fn with_data<U>(self, data: U) -> CoreLaunchInfo<U> {
        CoreLaunchInfo {
            core: self.core,
            rom: self.rom,
            bios: self.bios,
            files: self.files,
            save_state: self.save_state,
            data,
        }
    }
}
