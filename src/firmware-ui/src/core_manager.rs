use std::path::Path;
use std::time::SystemTime;

use byteorder::{LittleEndian, ReadBytesExt};
use tracing::info;

use mister_fpga::core::file::SdCard;
use mister_fpga::core::{MenuCore, MisterFpgaCore};
use mister_fpga::fpga::MisterFpga;
use one_fpga::core::SaveState;
use one_fpga::runner::{CoreLaunchInfo, CoreType, Slot};
use one_fpga::{Core, OneFpgaCore};

pub struct CoreManager {
    fpga: MisterFpga,
    current_core: Option<OneFpgaCore>,
}

impl CoreManager {
    pub fn new(fpga: MisterFpga) -> Self {
        Self {
            fpga,
            current_core: None,
        }
    }

    pub fn fpga(&self) -> &MisterFpga {
        &self.fpga
    }

    pub fn fpga_mut(&mut self) -> &mut MisterFpga {
        &mut self.fpga
    }

    /// Create a core for the current FPGA configuration.
    fn create_core(&mut self, is_menu: bool) -> Result<OneFpgaCore, String> {
        let mut core = if is_menu {
            MenuCore::new(self.fpga.clone())
                .map(OneFpgaCore::new)
                .map_err(|e| format!("Could not instantiate Core: {e}"))?
        } else {
            MisterFpgaCore::new(self.fpga.clone())
                .map(OneFpgaCore::new)
                .map_err(|e| format!("Could not instantiate Core: {e}"))?
        };

        core.init().map_err(|e| e.to_string())?;
        core.set_volume(255).map_err(|e| e.to_string())?;
        core.set_rtc(SystemTime::now()).map_err(|e| e.to_string())?;

        self.current_core = Some(core.clone());
        Ok(core)
    }

    pub fn load(&mut self, program: &[u8], is_menu: bool) -> Result<OneFpgaCore, String> {
        let program = if &program[..6] != b"MiSTer" {
            program
        } else {
            let start = 16;
            let size = (&program[12..]).read_u32::<LittleEndian>().unwrap() as usize;
            &program[start..start + size]
        };

        self.fpga.wait_for_ready();
        self.fpga
            .load(program)
            .map_err(|e| format!("Could not load program: {e:?}"))?;
        self.fpga.core_reset();

        self.create_core(is_menu)
    }

    pub fn load_menu(&mut self) -> Result<OneFpgaCore, String> {
        info!("Loading menu");
        let bytes = include_bytes!("../assets/menu.rbf");
        let core = self.load(bytes, true)?;
        self.fpga_mut().osd_enable();
        Ok(core)
    }

    pub fn load_core(&mut self, path: impl AsRef<Path>) -> Result<OneFpgaCore, String> {
        info!("Loading core from: {:?}", path.as_ref().display());
        let bytes = std::fs::read(path.as_ref()).map_err(|e| e.to_string())?;
        let core = self.load(&bytes, false)?;
        Ok(core)
    }

    pub fn launch(&mut self, info: CoreLaunchInfo<()>) -> Result<OneFpgaCore, String> {
        let mut core = match info.core {
            CoreType::Current => self.get_current_core().ok_or("No core running")?,
            CoreType::Menu => self.load_menu()?,
            CoreType::RbfFile(path) => self.load_core(path)?,
        };

        let mister_core = core.as_any_mut().downcast_mut::<MisterFpgaCore>().unwrap();

        if let Some(rom) = &info.rom {
            mister_core
                .send_rom(rom.clone())
                .map_err(|e| e.to_string())?;
        }

        if !info.files.is_empty() {
            let should_sav = mister_core
                .menu_options()
                .iter()
                .filter_map(|x| x.as_load_file_info())
                .any(|i| i.save_support);

            if should_sav {
                for (idx, f) in info.files {
                    if let Slot::File(ref path) = f {
                        mister_core.mount(SdCard::from_path(path)?, idx as u8)?;
                    }
                }
            }
            mister_core.end_send_file()?;
            while mister_core.poll_mounts()? {}
        } else {
            mister_core.end_send_file()?;
        }

        // Load all savestates.
        if let Some(savestate_manager) = mister_core.save_states_mut() {
            for (slot, state) in savestate_manager.slots_mut().iter_mut().enumerate() {
                if let Some(Slot::File(path)) = info.save_state.get(slot) {
                    let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
                    state.load(&mut f).map_err(|e| e.to_string())?;
                }
            }
        }

        Ok(core)
    }

    pub fn get_current_core(&mut self) -> Option<OneFpgaCore> {
        self.current_core.clone()
    }

    pub fn show_osd(&mut self) {
        self.fpga_mut().osd_enable();
    }

    pub fn hide_osd(&mut self) {
        self.fpga_mut().osd_disable();
    }
}
