use crate::platform::de10::fpga::Fpga;
use byteorder::{LittleEndian, ReadBytesExt};
use std::path::Path;

pub mod core;
pub use core::FpgaCore;

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
    type Core<'a> = FpgaCore;

    fn load_program(&mut self, path: impl AsRef<Path>) -> Result<FpgaCore, String> {
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
        // self.fpga.core_reset().expect("Could not reset the Core");
        self.fpga.load_rbf(program).expect("Could not load RBF");
        self.fpga.core_reset().expect("Could not reset the Core");

        let core = FpgaCore::new(self.fpga.clone()).expect("Could not create a Core");
        self.fpga.wait_for_ready();
        self.hide_menu();

        unsafe {
            crate::file_io::FindStorage();
            crate::platform::de10::user_io::user_io_init(
                "\0".as_ptr() as *const _,
                std::ptr::null(),
            );
        }

        crate::platform::de10::osd::OsdSetSize(19);
        self.show_menu();

        Ok(core)
    }

    fn show_menu(&mut self) {
        self.fpga_mut().osd_enable();
    }

    fn hide_menu(&mut self) {
        self.fpga_mut().osd_disable();
    }
}
