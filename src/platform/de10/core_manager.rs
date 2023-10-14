use crate::platform::de10::fpga::Fpga;
use byteorder::{LittleEndian, ReadBytesExt};
use std::path::Path;

pub mod core;
pub use core::MisterFpgaCore;

pub struct CoreManager {
    fpga: Fpga,
}

impl CoreManager {
    pub fn new(fpga: Fpga) -> Self {
        Self { fpga }
    }

    pub fn fpga(&self) -> &Fpga {
        &self.fpga
    }

    pub fn fpga_mut(&mut self) -> &mut Fpga {
        &mut self.fpga
    }
}

impl crate::platform::CoreManager for CoreManager {
    type Core = MisterFpgaCore;

    fn load_program(&mut self, path: impl AsRef<Path>) -> Result<MisterFpgaCore, String> {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        let program = bytes.as_slice();

        if program.as_ptr() as usize % 4 != 0 {
            return Err("Program is not aligned to 4 bytes".to_string());
        }

        let program = if &program[..6] != b"MiSTer" {
            program
        } else {
            let start = 16;
            let size = (&program[12..]).read_u32::<LittleEndian>().unwrap() as usize;
            &program[start..start + size]
        };

        self.fpga.wait_for_ready();
        self.fpga
            .load_rbf(program)
            .map_err(|e| format!("Could not load program: {e:?}"))?;
        self.fpga
            .core_reset()
            .map_err(|_| "Could not reset the Core".to_string())?;

        let core = MisterFpgaCore::new(self.fpga.clone())
            .map_err(|e| format!("Could not instantiate Core: {e}"))?;
        self.hide_menu();

        unsafe {
            crate::file_io::FindStorage();
            crate::platform::de10::user_io::user_io_init(
                "\0".as_ptr() as *const _,
                std::ptr::null(),
            );
        }

        crate::platform::de10::osd::OsdSetSize(19);

        Ok(core)
    }

    fn show_menu(&mut self) {
        self.fpga_mut().osd_enable();
    }

    fn hide_menu(&mut self) {
        self.fpga_mut().osd_disable();
    }
}
